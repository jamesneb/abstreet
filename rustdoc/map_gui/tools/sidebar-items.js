initSidebarItems({"constant":[["NEXT_RELEASE",""]],"enum":[["Executable",""]],"fn":[["app_header","A standard way to group a home button back to the title screen, the title of the current app, and a button to change maps. Callers must handle the `change map` and `home` click events."],["change_map_btn","A button to change maps, with default keybindings"],["cmp_count","Less is better"],["cmp_dist","Shorter is better"],["cmp_duration","Shorter is better"],["draw_isochrone","Thresholds are Durations, in units of seconds"],["find_exe","Returns the path to an executable. Native-only."],["goal_marker","Draw a goal marker pointing at something."],["grey_out_map","Make it clear the map can’t be interacted with right now."],["home_btn","A button to return to the title screen"],["intersections_from_roads",""],["loading_tips",""],["make_heatmap",""],["nice_country_name",""],["nice_map_name",""],["open_browser",""],["percentage_bar",""],["prompt_to_download_missing_data","Prompt to download a missing city. On either success or failure (maybe the player choosing to not download, maybe a network error), the new map isn’t automatically loaded or anything; up to the caller to handle that."],["start_marker","Draw a start marker pointing at something."],["version","Returns the version of A/B Street to link to. When building for a release, this points to that new release. Otherwise it points to the current dev version."]],"mod":[["camera",""],["city_picker",""],["colors",""],["command",""],["heatmap",""],["icons",""],["importer",""],["labels",""],["minimap",""],["navigate",""],["title_screen",""],["trip_files",""],["turn_explorer",""],["ui","Generic UI tools. Some of this should perhaps be lifted to widgetry."],["updater",""],["url",""],["waypoints",""]],"struct":[["CameraState","Represents the state of a widgetry Canvas."],["ChooseSomething","Choose something from a menu, then feed the answer to a callback."],["CityPicker","Lets the player switch maps."],["ColorDiscrete",""],["ColorLegend",""],["ColorNetwork",""],["ColorScale",""],["DefaultMap","Track the last map used, to resume next session."],["DivergingScale",""],["DrawRoadLabels","Labels roads when unzoomed. Label size and frequency depends on the zoom level."],["FilePicker",""],["Grid","A 2D grid containing some arbitrary data."],["HeatmapOptions",""],["InputWaypoints","Click to add waypoints, drag them, see the list on a panel and delete them. The caller owns the Panel and the World, since there’s probably more stuff there too."],["Minimap",""],["Navigator",""],["PopupMsg","Display a message dialog."],["PromptInput","Prompt for arbitrary text input, then feed the answer to a callback."],["RunCommand","Executes a command and displays STDOUT and STDERR in a loading screen window. Only works on native, of course."],["TitleScreen","A title screen shared among all of the A/B Street apps."],["TripManagement","Save sequences of waypoints as named trips. Basic file management – save, load, browse. This is useful to define “test cases,” then edit the bike network and “run the tests” to compare results."],["TurnExplorer","A tool to explore all of the turns from a single lane."],["URLManager","Utilities for reflecting the current map and viewport in the URL on the web. No effect on native."],["WaypointID",""]],"trait":[["MinimapControls","Customize the appearance and behavior of a minimap."],["TripManagementState",""]]});