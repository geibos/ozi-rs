## ADDED Requirements

### Requirement: The inspector shows time on the move

For a track whose points carry timestamps, the track inspector SHALL show the
time the crew was moving beside the span from first point to last.

The crew SHALL count as stopped wherever they moved less than the stop
distance within the stop window — by default 25 m in two minutes, both set on
the settings screen. A standing navigator's GPS position wanders by five to
twenty metres, so a threshold per fix would count a rest stop as slow walking;
a threshold over a window does not. Slow, careful combing at one kilometre an
hour SHALL still count as moving under the defaults.

A track without timestamps SHALL show no moving time.

#### Scenario: A walk with a rest stop

- **WHEN** a track walks for half an hour, stands for twenty minutes with its position wandering within ten metres, and walks for another half hour
- **THEN** the time on the move reads about an hour, and the span about an hour and twenty minutes

#### Scenario: Combing slowly

- **WHEN** a track moves steadily at one kilometre an hour for an hour
- **THEN** the whole hour counts as moving

#### Scenario: A recording that never moved

- **WHEN** a track's positions wander within ten metres for half an hour
- **THEN** the time on the move is zero

#### Scenario: A different stop threshold

- **WHEN** the operator raises the stop distance on the settings screen
- **THEN** the inspector's time on the move is recomputed with it

### Requirement: The inspector shows ascent and descent

For a track whose points carry elevations, the track inspector SHALL show the
total ascent and the total descent along it, counting a rise or a fall only
once it reaches the climb threshold — by default 5 m, set on the settings
screen. A top or a bottom SHALL count once the elevation has come back from it
by the threshold, so that the last metres of a climb are not lost.

Summed point by point, GPS elevation noise of a few metres adds up to a
climb several times the real one; a threshold keeps the noise out and the
hills in.

A track without elevations SHALL show no ascent or descent.

#### Scenario: Noise on flat ground

- **WHEN** a track's elevations wander two metres either side of 100 m
- **THEN** ascent and descent both read 0 m

#### Scenario: A hill

- **WHEN** a track climbs steadily from 100 m to 200 m and comes back down
- **THEN** ascent and descent each read about 100 m
