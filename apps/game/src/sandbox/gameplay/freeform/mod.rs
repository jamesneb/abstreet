mod area_spawner;
#[cfg(not(target_arch = "wasm32"))]
mod importers;
mod spawner;

use rand::seq::SliceRandom;
use rand::Rng;

use abstutil::Timer;
use geom::{Distance, Duration};
use map_gui::tools::{grey_out_map, CityPicker, PromptInput};
use map_gui::ID;
use map_model::{IntersectionID, Position};
use sim::rand_dist;
use synthpop::{IndividTrip, PersonSpec, Scenario, TripEndpoint, TripMode, TripPurpose};
use widgetry::tools::{open_browser, PopupMsg};
use widgetry::{
    lctrl, EventCtx, GfxCtx, HorizontalAlignment, Key, Line, Outcome, Panel, SimpleState, State,
    Text, VerticalAlignment, Widget,
};

use crate::app::{App, Transition};
use crate::common::jump_to_time_upon_startup;
use crate::edit::EditMode;
use crate::sandbox::gameplay::{GameplayMode, GameplayState};
use crate::sandbox::{Actions, SandboxControls, SandboxMode};

pub struct Freeform {
    top_right: Panel,
}

impl Freeform {
    pub fn new_state(ctx: &mut EventCtx, app: &App) -> Box<dyn GameplayState> {
        map_gui::tools::update_url_map_name(app);

        Box::new(Freeform {
            top_right: Panel::empty(ctx),
        })
    }
}

impl GameplayState for Freeform {
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        app: &mut App,
        _: &mut SandboxControls,
        _: &mut Actions,
    ) -> Option<Transition> {
        match self.top_right.event(ctx) {
            Outcome::Clicked(x) => match x.as_ref() {
                "change map" => Some(Transition::Push(CityPicker::new_state(
                    ctx,
                    app,
                    Box::new(|_, app| {
                        let sandbox = if app.opts.dev {
                            SandboxMode::async_new(
                                app,
                                GameplayMode::Freeform(app.primary.map.get_name().clone()),
                                jump_to_time_upon_startup(Duration::hours(6)),
                            )
                        } else {
                            SandboxMode::simple_new(
                                app,
                                GameplayMode::Freeform(app.primary.map.get_name().clone()),
                            )
                        };
                        Transition::Multi(vec![Transition::Pop, Transition::Replace(sandbox)])
                    }),
                ))),
                "change scenario" => Some(Transition::Push(ChangeScenario::new_state(
                    ctx, app, "none",
                ))),
                "edit map" => Some(Transition::Push(EditMode::new_state(
                    ctx,
                    app,
                    GameplayMode::Freeform(app.primary.map.get_name().clone()),
                ))),
                "Start a new trip" => Some(Transition::Push(spawner::AgentSpawner::new_state(
                    ctx, app, None,
                ))),
                "Spawn area traffic" => {
                    Some(Transition::Push(area_spawner::AreaSpawner::new_state(ctx)))
                }
                "Record trips as a scenario" => Some(Transition::Push(PromptInput::new_state(
                    ctx,
                    "Name this scenario",
                    String::new(),
                    Box::new(|name, ctx, app| {
                        if abstio::file_exists(abstio::path_scenario(
                            app.primary.map.get_name(),
                            &name,
                        )) {
                            Transition::Push(PopupMsg::new_state(
                                ctx,
                                "Error",
                                vec![format!(
                                    "A scenario called \"{}\" already exists, please pick another \
                                     name",
                                    name
                                )],
                            ))
                        } else {
                            app.primary
                                .sim
                                .generate_scenario(&app.primary.map, name)
                                .save();
                            Transition::Pop
                        }
                    }),
                ))),
                _ => unreachable!(),
            },
            _ => None,
        }
    }

    fn draw(&self, g: &mut GfxCtx, _: &App) {
        self.top_right.draw(g);
    }

    fn recreate_panels(&mut self, ctx: &mut EventCtx, app: &App) {
        let rows = vec![
            Widget::custom_row(vec![
                Line("Sandbox")
                    .small_heading()
                    .into_widget(ctx)
                    .margin_right(18),
                map_gui::tools::change_map_btn(ctx, app).margin_right(8),
                ctx.style()
                    .btn_popup_icon_text("system/assets/tools/calendar.svg", "none")
                    .hotkey(Key::S)
                    .build_widget(ctx, "change scenario")
                    .margin_right(8),
                ctx.style()
                    .btn_outline
                    .icon_text("system/assets/tools/pencil.svg", "Edit map")
                    .hotkey(lctrl(Key::E))
                    .build_widget(ctx, "edit map")
                    .margin_right(8),
            ])
            .centered(),
            Widget::row(vec![
                ctx.style()
                    .btn_outline
                    .text("Start a new trip")
                    .build_def(ctx),
                /*ctx.style()
                .btn_outline
                .text("Spawn area traffic")
                .hotkey(Key::A)
                .build_def(ctx),*/
                ctx.style()
                    .btn_outline
                    .text("Record trips as a scenario")
                    .build_def(ctx),
            ])
            .centered(),
            Text::from_all(vec![
                Line("Select an intersection and press "),
                Key::Z.txt(ctx),
                Line(" to start traffic nearby"),
            ])
            .into_widget(ctx),
        ];

        self.top_right = Panel::new_builder(Widget::col(rows))
            .aligned(HorizontalAlignment::Right, VerticalAlignment::Top)
            .build(ctx);
    }
}

pub struct ChangeScenario;

impl ChangeScenario {
    pub fn new_state(ctx: &mut EventCtx, app: &App, current_scenario: &str) -> Box<dyn State<App>> {
        // (Button action, label, full description)
        let mut choices = Vec::new();
        for name in abstio::list_all_objects(abstio::path_all_scenarios(app.primary.map.get_name()))
        {
            if name == "weekday" {
                choices.push((
                    name,
                    "typical weekday traffic".to_string(),
                    "Trips will begin throughout the entire day. Midnight is usually quiet, so \
                     you may need to fast-forward to morning rush hour. Data comes from Puget \
                     Sound Regional Council's Soundcast model from 2014.",
                ));
            } else if name == "background" || name == "base_with_bg" {
                choices.push((
                    name,
                    "typical weekday traffic".to_string(),
                    "Home-to-work trips from 2011 UK census data are simulated. Traffic usually \
                     starts around 7am.",
                ));
            } else {
                choices.push((
                    name.clone(),
                    name,
                    "This is custom scenario data for this map",
                ));
            }
        }
        choices.push((
            "home_to_work".to_string(),
            "trips between home and work".to_string(),
            "Randomized people will leave homes in the morning, go to work, then return in the \
             afternoon. It'll be very quiet before 7am and between 10am to 5pm. The population \
             size and location of homes and workplaces is all guessed just from OpenStreetMap \
             tags.",
        ));
        choices.push((
            "random".to_string(),
            "random unrealistic trips".to_string(),
            "A fixed number of trips will start at midnight, but not constantly appear through \
             the day.",
        ));
        let country = &app.primary.map.get_name().city.country;
        // Until we add in census data for other countries, offering the option doesn't make sense.
        // Include "zz", used for one-shot imports, since we have no idea where those are located.
        if country == "us" || country == "zz" {
            choices.push((
                "census".to_string(),
                "generate from US census data".to_string(),
                "A population from 2010 US census data will travel between home and workplaces. \
                 Generating it will take a few moments as some data is downloaded for this map.",
            ));
        }
        choices.push((
            "none".to_string(),
            "none, except for buses".to_string(),
            "You can manually spawn traffic around a single intersection or by using the tool in \
             the top panel to start individual trips.",
        ));
        if cfg!(not(target_arch = "wasm32")) {
            choices.push((
                "import grid2demand".to_string(),
                "import Grid2Demand data".to_string(),
                "Select an input_agents.csv file from https://github.com/asu-trans-ai-lab/grid2demand"));
            choices.push((
                "import json".to_string(),
                "import JSON scenario".to_string(),
                "Select a JSON file specified by https://a-b-street.github.io/docs/tech/dev/formats/scenarios.html"));
        }

        let mut col = vec![
            Widget::row(vec![
                Line("Pick your scenario").small_heading().into_widget(ctx),
                ctx.style().btn_close_widget(ctx),
            ]),
            Line("Each scenario determines what people live and travel around this map")
                .into_widget(ctx),
        ];
        for (name, label, description) in choices {
            let btn = if name == current_scenario {
                ctx.style().btn_tab.text(label).disabled(true)
            } else {
                ctx.style().btn_outline.text(label)
            };
            col.push(
                Widget::row(vec![
                    btn.build_widget(ctx, name),
                    Text::from(Line(description).secondary())
                        .wrap_to_pct(ctx, 40)
                        .into_widget(ctx)
                        .align_right(),
                ])
                .margin_above(30),
            );
        }
        col.push(
            ctx.style()
                .btn_plain
                .btn()
                .label_underlined_text("Learn how to import your own data.")
                .build_def(ctx),
        );

        <dyn SimpleState<_>>::new_state(
            Panel::new_builder(Widget::col(col)).build(ctx),
            Box::new(ChangeScenario),
        )
    }
}

impl SimpleState<App> for ChangeScenario {
    fn on_click(
        &mut self,
        ctx: &mut EventCtx,
        app: &mut App,
        x: &str,
        _: &mut Panel,
    ) -> Transition {
        if x == "close" {
            Transition::Pop
        } else if x == "Learn how to import your own data." {
            open_browser(
                "https://a-b-street.github.io/docs/tech/trafficsim/travel_demand.html#custom-import",
            );
            Transition::Keep
        } else if x == "import grid2demand" {
            #[cfg(not(target_arch = "wasm32"))]
            {
                importers::import_grid2demand(ctx)
            }
            #[cfg(target_arch = "wasm32")]
            {
                // Silence compiler warnings
                let _ = ctx;
                unreachable!()
            }
        } else if x == "import json" {
            #[cfg(not(target_arch = "wasm32"))]
            {
                importers::import_json(ctx)
            }
            #[cfg(target_arch = "wasm32")]
            {
                // Silence compiler warnings
                let _ = ctx;
                unreachable!()
            }
        } else {
            Transition::Multi(vec![
                Transition::Pop,
                Transition::Replace(SandboxMode::simple_new(
                    app,
                    if x == "none" {
                        GameplayMode::Freeform(app.primary.map.get_name().clone())
                    } else {
                        GameplayMode::PlayScenario(
                            app.primary.map.get_name().clone(),
                            x.to_string(),
                            Vec::new(),
                        )
                    },
                )),
            ])
        }
    }

    fn draw(&self, g: &mut GfxCtx, app: &App) {
        grey_out_map(g, app);
    }
}

pub fn spawn_agents_around(i: IntersectionID, app: &mut App) {
    let map = &app.primary.map;
    let mut rng = app.primary.current_flags.sim_flags.make_rng();
    let mut scenario = Scenario::empty(map, "one-shot");

    if map.all_buildings().is_empty() {
        println!("No buildings, can't pick destinations");
        return;
    }

    let mut timer = Timer::new(format!(
        "spawning agents around {} (rng seed {:?})",
        i, app.primary.current_flags.sim_flags.rng_seed
    ));

    for l in &map.get_i(i).incoming_lanes {
        let lane = map.get_l(*l);
        if lane.is_driving() || lane.is_biking() {
            for _ in 0..10 {
                let mode = if rng.gen_bool(0.7) && lane.is_driving() {
                    TripMode::Drive
                } else {
                    TripMode::Bike
                };
                scenario.people.push(PersonSpec {
                    orig_id: None,
                    trips: vec![IndividTrip::new(
                        app.primary.sim.time(),
                        TripPurpose::Shopping,
                        TripEndpoint::SuddenlyAppear(Position::new(
                            lane.id,
                            rand_dist(&mut rng, Distance::ZERO, lane.length()),
                        )),
                        TripEndpoint::Building(map.all_buildings().choose(&mut rng).unwrap().id),
                        mode,
                    )],
                });
            }
        } else if lane.is_walkable() {
            for _ in 0..5 {
                scenario.people.push(PersonSpec {
                    orig_id: None,
                    trips: vec![IndividTrip::new(
                        app.primary.sim.time(),
                        TripPurpose::Shopping,
                        TripEndpoint::SuddenlyAppear(Position::new(
                            lane.id,
                            rand_dist(&mut rng, 0.1 * lane.length(), 0.9 * lane.length()),
                        )),
                        TripEndpoint::Building(map.all_buildings().choose(&mut rng).unwrap().id),
                        TripMode::Walk,
                    )],
                });
            }
        }
    }

    let retry_if_no_room = false;
    app.primary.sim.instantiate_without_retries(
        &scenario,
        map,
        &mut rng,
        retry_if_no_room,
        &mut timer,
    );
    app.primary.sim.tiny_step(map, &mut app.primary.sim_cb);
}

pub fn actions(_: &App, id: ID) -> Vec<(Key, String)> {
    match id {
        ID::Building(_) => vec![(Key::Z, "start a trip here".to_string())],
        ID::Intersection(_) => vec![(Key::Z, "spawn agents here".to_string())],
        _ => Vec::new(),
    }
}

pub fn execute(ctx: &mut EventCtx, app: &mut App, id: ID, action: &str) -> Transition {
    match (id, action) {
        (ID::Building(b), "start a trip here") => {
            Transition::Push(spawner::AgentSpawner::new_state(ctx, app, Some(b)))
        }
        (ID::Intersection(id), "spawn agents here") => {
            spawn_agents_around(id, app);
            Transition::Keep
        }
        _ => unreachable!(),
    }
}
