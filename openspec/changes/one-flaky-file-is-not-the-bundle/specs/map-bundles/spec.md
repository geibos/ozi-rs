## ADDED Requirements

### Requirement: A dropped transfer is retried before the file is given up

A file within a bundle SHALL be fetched again after a failed transfer, up to a
bounded number of attempts, before the download fails. A bundle is fetched over
the link a crew has in the field, where a dropped connection is ordinary, and
one such file SHALL NOT cost them the whole bundle.

A cancelled transfer SHALL NOT be retried: cancellation is the operator's
instruction, not a transport failure.

A retry SHALL be reported, so that it is not mistaken for a stall.

A retry SHALL continue from what the failed attempt already wrote rather than
fetching the file again, unless the server does not honour the request for the
remainder, in which case the file SHALL be fetched again from the start. What
a failed attempt wrote SHALL be kept for the next one and SHALL be removed once
the file is given up on.

#### Scenario: A connection drops partway through a bundle

- **WHEN** one file's transfer fails and the next attempt succeeds
- **THEN** that file lands complete and the bundle download continues

#### Scenario: The operator cancels

- **WHEN** a transfer ends because the download was cancelled
- **THEN** it is not attempted again

#### Scenario: A drop near the end of a large map

- **WHEN** a transfer fails after most of a file has been written and the next attempt is made
- **THEN** the remainder is requested and the file is completed without fetching what had already arrived

#### Scenario: The file is given up on

- **WHEN** every attempt at a file has failed
- **THEN** nothing partial is left behind for it

#### Scenario: Watching a retry

- **WHEN** a file is being fetched again after a failure
- **THEN** the progress reported says so, naming the file and the attempt
