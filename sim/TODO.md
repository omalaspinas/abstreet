# What should we add

* How to treat people that are off-map (how do they transition there? could we model transition/quarantine/hosp outside the city? do we have the info?).
* Add event-based diffusion of illness on sidewalks (N possibilities):
  1. Use `peds_per_traversable` (mapping from lane/turn to pedestrians) and assign a small transmission probability (easy).
  2. Track pedestrians entering/leaving a particular traversable (like buildings), `AgentEntersTraversable` need to add `AgentLeavesTraversable`:
     * if they cross each other one can have transmission (they enter traversable in opposite order)
     * and/or if they follow each other they also can transmit (the enter traversable in the same order).
     * we could add a memory that last for some time that stores the position of the pedestrian for some time to simulate virus emission.
  * 
  * En
* 
* Map of Geneva


The heatmap is in the UI layer, it sounds like this should live in the sim crate. mechanics/walking.rs has really simple state. peds_per_traversable is a mapping from Traversable::Lane (sidewalks only) or Traversable::Turn to list of pedestrians there

How about in PandemicModel, you store some new struct per LaneID (just for the sidewalks). You know when a pedestrian enters a sidewalk by listening to AgentEntersTraversable. If you need to know when they leave a sidewalk, we can add AgentLeavesTraversable