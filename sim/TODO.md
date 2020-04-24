# What should we add

* How to treat people that are off-map (how do they transition there? could we model transition/quarantine/hosp outside the city? do we have the info?).
* Add event-based diffusion of illness on sidewalks (N possibilities):
  1. Use `peds_per_traversable` (mapping from lane/turn to pedestrians) and assign a small transmission probability (easy).
  2. Track pedestrians entering/leaving a particular traversable (like buildings), `AgentEntersTraversable` need to add `AgentLeavesTraversable`:
     * if they cross each other one can have transmission (they enter traversable in opposite order)
     * and/or if they follow each other they also can transmit (the enter traversable in the same order).
     * we could add a memory that last for some time that stores the position of the pedestrian for some time to simulate virus emission.
  3. From (2) data we could be more precise and interpolate between beginning/end.
* From OSM get a map of Geneva
* Make sim ouf sound cast run.