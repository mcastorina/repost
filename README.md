# repost
`repost` is an interpreter to easily define and send HTTP requests for multiple environments.

## Usage
Repost utilizes an interpreter environment to make creating and
sending requests easier. Repost has context specific commands, and
you can always see which environment or request you are using
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
In general, repost simplifies this flow:
```
(modify input) -> (send request) -> (extract output)
```

Each request has **input options** and **output options**. Input
options are defined when creating the request by using `{name}` -- they
can be used anywhere in the url, headers, or body. The name inside
the `{}` correlates to a variable, and the option will automatically be
populated if the variable exists for the current environment. Output options
are added as **extractors**. Extractors will extract a header value
or a part of a JSON body and save it in a variable.


## Features

| Status             | Feature description                              |
|:------------------:|--------------------------------------------------|
| :white_check_mark: | create request / variable                        |
| :white_check_mark: | show tables with formatting                      |
| :white_check_mark: | run request                                      |
| :white_check_mark: | input option substitution                        |
| :white_check_mark: | output option extraction                         |
| :white_check_mark: | automatically set input option to variable value |
| :white_check_mark: | edit variable                                    |
| :soon:             | edit request                                     |
| :soon:             | tab completion                                   |
|                    | extract from all items in an array               |
|                    | extract from other data formats                  |
|                    | option to hide variable values                   |
|                    | run flags                                        |
|                    | clipboard integration                            |
|                    | create request from curl command                 |
|                    | save responses                                   |
|                    | search command                                   |
| :question:         | variable generation                              |
| :question:         | dependency graph                                 |
|                    | global environment                               |
|                    | color requests that have all options satisfied   |

* :white_check_mark: -- In master
* :soon: -- In progress
* :question: -- Might not happen
* Blank -- Rough idea
