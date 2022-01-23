initSidebarItems({"enum":[["Spot",""],["WalkingNode",""]],"fn":[["all_vehicle_costs_from","Starting from some initial spot, calculate the cost to all buildings. If a destination isn’t reachable, it won’t be included in the results. Ignore results greater than the time_limit away."],["all_walking_costs_from","Starting from some initial buildings, calculate the cost to all others. If a destination isn’t reachable, it won’t be included in the results. Ignore results greater than the time_limit away."],["find_scc","Calculate the strongly connected components (SCC) of the part of the map accessible by constraints (ie, the graph of sidewalks or driving+bike lanes). The largest component is the “main” graph; the rest is disconnected. Returns (lanes in the largest “main” component, all other disconnected lanes)"],["vehicle_cost","This returns the pathfinding cost of crossing one road and turn, in units of time. It factors in the ideal time to cross the space and penalties for entering an access-restricted zone, taking an unprotected turn, or going up a steep hill for some vehicle types. If this returns `None`, then the movement isn’t actually allowed."]],"mod":[["walking",""]],"struct":[["Item",""],["WalkingOptions",""]]});