# repost
`repost` is a tool to easily define and send HTTP requests.

## Usage
Repost utilizes an interpreter environment to make creating and
sending requests easier. Repost has context specific commands and
you can always see which environment or request you are editing
from the command prompt.

Execute `repost` to start the session. All information is saved in
a sqlite database in `$XDG_CONFIG_DIR/repost/$WORKSPACE_NAME.db`.

### Getting started
This section shows how to create a request, define variables, and
add extractors.

```
[repost] > use workspace example
[example] > create request get-example {host}/data.json
[example] > create variable host local=http://localhost:8080 stage=https://stage.example.com
[example] > use environment stage
[example][stage] > use request get-example
[example][stage][get-example] > show options

  +--------------+-------------+---------------------------+
  | request_name | option_name | value                     |
  +--------------+-------------+---------------------------+
  | get-example  | host        | https://stage.example.com |
  +--------------+-------------+---------------------------+

[example][stage][get-example] > use environment local
[example][local][get-example] > show options

  +--------------+-------------+-----------------------+
  | request_name | option_name | value                 |
  +--------------+-------------+-----------------------+
  | get-example  | host        | http://localhost:8080 |
  +--------------+-------------+-----------------------+

[example][local][get-example] > run
> GET http://localhost:8080/data.json

< 200 OK
< server: SimpleHTTP/0.6 Python/3.7.6
< date: Tue, 23 Jun 2020 16:19:51 GMT
< content-type: application/json
< content-length: 101
< last-modified: Tue, 23 Jun 2020 16:19:42 GMT

{
  "id": "abcde",
  "name": "repost",
  "samples": [
    {
      "id": "1",
      "value": "a"
    },
    {
      "id": "2",
      "value": "b"
    }
  ]
}
[example][local][get-example] > extract body id --to-var id
[example][local][get-example] > info

      Name:  get-example
    Method:  GET
       URL:  {host}/data.json
   Headers:
     Body?:  false

  Input Options
  +------+-----------------------+
  | name | current value         |
  +------+-----------------------+
  | host | http://localhost:8080 |
  +------+-----------------------+

  Output Options
  +-----------------+------+--------+
  | output variable | type | source |
  +-----------------+------+--------+
  | id              | body | id     |
  +-----------------+------+--------+

[example][local][get-example] > run
> GET http://localhost:8080/data.json

< 200 OK
< server: SimpleHTTP/0.6 Python/3.7.6
< date: Tue, 23 Jun 2020 16:20:34 GMT
< content-type: application/json
< content-length: 101
< last-modified: Tue, 23 Jun 2020 16:19:42 GMT

{
  "id": "abcde",
  "name": "repost",
  "samples": [
    {
      "id": "1",
      "value": "a"
    },
    {
      "id": "2",
      "value": "b"
    }
  ]
}

id => abcde
[example][local][get-example] > show variables

  +-------+------+-------------+-----------------------+-------------+-------------------------+
  | rowid | name | environment | value                 | source      | timestamp               |
  +-------+------+-------------+-----------------------+-------------+-------------------------+
  | 1     | host | local       | http://localhost:8080 | user        | 2020-06-23 16:17:06 UTC |
  | 3     | id   | local       | abcde                 | get-example | 2020-06-23 16:20:34 UTC |
  +-------+------+-------------+-----------------------+-------------+-------------------------+

```

## Design
There are two main resources managed in `repost`: **requests** and **variables**.

### Requests
A "request" is referring to a single named HTTP request. It has the following attributes:

* **Name:** The name of the request
* **Method:** HTTP method (GET, POST, etc.)
* **URL:** The target of the HTTP request
* **Headers:** HTTP request headers
* **Body:** HTTP request body (optional)
* **Variables:** Internally managed list of variables used in this request

The most interesting here is **variables**. Variables can be used
in any part of the request (except name and method) and are denoted using
`{variable_name}` (e.g. **URL:** `{host}/health`).

### Variables
A "variable" is a way to parameterize a request. It has the following attributes:

* **Name:** The name of the variable
* **Value:** A list of possible values to pull from (see below for explanation)
* **Generator:** The way to generate the value for this variable

As stated above, **values** are a list of possible values to use.
One of the key features of `repost` is to easily send the same
requests to different environments. This is done using variables
with environment specific values. Environments are user defined,
and `repost` aims to provide tools to clearly see what variables
are required for requests and whether they are satisfied for certain
environments.

The **generator** defines how to generate the value for the variable.
There are three types of generators:

* **Constant Generator:** Constant, environment specific values
* **Script Generator:** Generated via shell script
* **Request Generator:** Generated via another `repost` request (in the same environment)
