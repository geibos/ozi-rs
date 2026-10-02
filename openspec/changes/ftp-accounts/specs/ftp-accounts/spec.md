## ADDED Requirements

### Requirement: Bundles and results have separate FTP accounts

The system SHALL keep FTP accounts in two roles: at most one account for
downloading bundles, and any number of endpoints for uploading results, each
with its own host, port, login and folder. An account SHALL carry a name the
operator chooses, so that the result endpoints can be told apart.

Bundles come from one place under one account; results go back to several
contours, each under an account of its own (owner, 2026-10-01).

#### Scenario: Four result endpoints

- **WHEN** the operator adds four result endpoints with different hosts and logins
- **THEN** all four are listed under results, each with its own login, and none of them is the bundle account

#### Scenario: A second bundle account

- **WHEN** a bundle account exists and another is saved in the bundle role
- **THEN** the second is refused and the first is unchanged

#### Scenario: An account that cannot be reached by name

- **WHEN** the operator saves an account with no host, no login or no name, or a host with a path or spaces in it
- **THEN** the account is refused with a message naming the field, and nothing is saved

### Requirement: A password lives only in the credential store

The system SHALL store an FTP password in the operating system's credential
store — Keychain on macOS, Credential Manager on Windows — and SHALL NOT write
it to the accounts file, the project, the session or the logs. The accounts
file SHALL hold the host, port, login, folder, name and role.

Saving an account without a password SHALL leave its stored password as it
was; deleting an account SHALL delete its password.

#### Scenario: What is on disk

- **WHEN** an account is saved with a password
- **THEN** the accounts file contains its host and login and does not contain the password, and the account is listed as having one

#### Scenario: Changing the host only

- **WHEN** the operator changes an account's host and leaves the password field empty
- **THEN** the stored password is kept

#### Scenario: Deleting an account

- **WHEN** the operator deletes an account
- **THEN** it is gone from the list and its password from the credential store

### Requirement: An account can be checked

The system SHALL offer to check an account: connect to its host, log in with
its stored password, and change to its folder. The answer SHALL say which of
these failed — the server could not be reached, the login was refused, the
folder does not exist, no password is stored — and SHALL show the server's own
reply beside it. A server that does not answer SHALL be given up on within
seconds rather than hang the check.

#### Scenario: Everything works

- **WHEN** the operator checks an account whose host, login, password and folder are right
- **THEN** the check reports success and the folder the server says it is in

#### Scenario: A wrong password

- **WHEN** the server answers the password with 530
- **THEN** the check reports the login was refused, with the server's text

#### Scenario: A folder that is not there

- **WHEN** the login succeeds and the folder does not exist
- **THEN** the check reports the folder is missing

#### Scenario: A server that accepts the connection and says nothing

- **WHEN** the host accepts the connection and never sends its greeting
- **THEN** the check gives up within its time limit and reports the server could not be reached

### Requirement: A damaged accounts file is not overwritten

When the accounts file cannot be read, the system SHALL report it and SHALL
refuse to save over it, so that a file the operator can still repair by hand
is not replaced by an empty list.

#### Scenario: A file that is not JSON

- **WHEN** the accounts file is damaged and the operator opens the FTP settings
- **THEN** the screen says the file cannot be read and where it is, and saving an account is refused
