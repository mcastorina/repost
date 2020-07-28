# repost
`repost` is an interpreter to easily define and send HTTP requests for multiple environments.

## Who should use this?
This tool is targeted for developers who send various different
HTTP requests for specific environments (local, staging, production).
If you find yourself having to mentally keep track of what values
are in the bash variables to your curl command, this tool is for
you.

## Key features

* named requests are easy to run
* request specific input options
* environment specific variables automatically populate input options
* extract variables from responses (and use in other requests!)
* easily send many requests

## Design
Repost utilizes an interpreter environment to make creating and
sending multiple requests easier. The first thing to know is that
repost will display the current workspace, environment, and request
on the command line. The environment and request are optional and
may not be displayed.

```
[repost][local][get-example] >
 ^^^^^^  ^^^^^  ^^^^^^^^^^^
    \       \           \___ request
     \       \______________ environment
      \_____________________ workspace
```

Related to these values, there are two distinct states that the
interpreter can be in: base and request. When request is set, you
will have access to request specific commands.

Another important thing to know is input options are denoted by
`{name}` and can be anywhere in the url, headers, or body.

## Installation
The binary can be downloaded from the release page.

Alternatively, you may build from source (rust build tools required):

```
» git clone https://github.com/mcastorina/repost && cd repost
» make release
» ls -l ./target/release/repost
```

## Quick start
This section shows how to create a request, define variables, and
add extractors.

* [Setting a workspace](#setting-a-workspace)
* [Create a request](#create-a-request)
* [Set options](#set-options)
* [Run a request](#run-a-request)
* [Define a variable](#define-a-variable)
* [Add extractors](#add-extractors)
  * [JSON query expression](#json-query-expression)

Execute `repost` to start the session. All information is saved in
a sqlite database in `$XDG_CONFIG_DIR/repost/$WORKSPACE_NAME.db`
or `$HOME/.repost/$WORKSPACE_NAME.db`.

**Note:** If you forget what command does what, use `help` or `--help`
for more information about the available commands and flags.

### Setting a workspace
Your current workspace is where all of your data will be saved.
Repost starts with the default workspace: `repost`, but we can
change that using `set workspace <workspace_name>`.

```
[repost] > set workspace example
[example] >
```

To show the available workspaces, use `show workspaces`.

```
[example] > show workspaces
  +-----------+
  | workspace |
  +-----------+
  | example   |
  | repost    |
  +-----------+
```

### Create a request
The minimum request has a name and a URL. Headers can be added with
`-H` and a body with `-d`. If the argument to `-d` starts with `@`,
repost will try to find the file to use as its body. Use `{option-name}`
anywhere you want to use an input option. The method may be inferred
from the request name or manually set with `-m`.

This example creates a request with one input option named `host`.
```
[example] > create request get-data http://{host}/data.json
```

To get more information about the request we just made, let's set it as
our current request and view the `info`.

```
[example] > set request get-data
[example][get-data] > info

      Name:  get-data
    Method:  GET
       URL:  http://{host}/data.json
   Headers:
     Body?:  false

  Input Options
  +------+----------------+
  | name | current values |
  +------+----------------+
  | host |                |
  +------+----------------+

  Planned Requests
  +------+--------+-----+---------+-------+
  | name | method | url | headers | body? |
  +------+--------+-----+---------+-------+
  +------+--------+-----+---------+-------+

```

### Set options
From the request state, we can use `set option` to set the value for the request.

```
[example][get-data] > set option host localhost:8000
[example][get-data] > info

      Name:  get-data
    Method:  GET
       URL:  http://{host}/data.json
   Headers:
     Body?:  false

  Input Options
  +------+----------------+
  | name | current values |
  +------+----------------+
  | host | localhost:8000 |
  +------+----------------+

  Planned Requests
  +----------+--------+---------------------------------+---------+-------+
  | name     | method | url                             | headers | body? |
  +----------+--------+---------------------------------+---------+-------+
  | get-data | GET    | http://localhost:8000/data.json |         | false |
  +----------+--------+---------------------------------+---------+-------+

```

Here we see the current value, and the planned requests if we were
to run it. You may set multiple values for the same input option by
providing more values on the command line.

### Run a request
There are two ways to run a request. If you are in a request state, simply using
`run` will execute the current request. The other way is to specify the request
name to run.

```
[example][get-data] > run
> GET http://localhost:8000/data.json

< 200 OK
< server: SimpleHTTP/0.6 Python/3.6.7
< date: Thu, 23 Jul 2020 21:18:40 GMT
< content-type: application/json
< content-length: 97
< last-modified: Thu, 16 Jul 2020 04:03:18 GMT

{
  "id": "abcde",
  "name": "repost",
  "samples": [
    {
      "id": "id-1",
      "value": "a"
    },
    {
      "id": "id-2",
      "value": "b"
    }
  ]
}
```

### Define a variable
Variables are environment specific, and should generally match your
request's input options. When you are in an environment, the value
of the variable will automatically be populated in the input option.

The syntax is `create variable NAME environment=value environment=value ...`.

```
[example][get-data] > create variable host local=localhost:8000 stage=example.stage.com
[example][get-data] > show variables

  +----+------+-------------+-------------------+--------+
  | id | name | environment | value             | source |
  +----+------+-------------+-------------------+--------+
  | 1  | host | local       | localhost:8000    | user   |
  | 2  | host | stage       | example.stage.com | user   |
  +----+------+-------------+-------------------+--------+

```

Now we can set an environment using `set environment`. Additionally,
`show environments` will display all of the available environments.

```
[example][get-data] > set environment stage
[example][stage][get-data] >
```

Note that when we set the environment, our input option gets updated
to the value of the variable.

### Add extractors
Extractors may be added in the request state. The command `extract`
is used to add it as an output option to the request. Extractors will
try to capture a certain part of a request and save it to a variable.

The syntax is `extract TYPE SOURCE --to-var NAME`. `TYPE` is `body`
or `header` to denote which part of the response to extract from.
`SOURCE` depends on the `TYPE`: `header` is the header key and
`body` is a simplified JSON query expression (explained below).

Currently, only JSON extraction is supported.

#### JSON query expression
The simplified language is `.` separated sub-fields and `[]` for
accessing arrays. The value inside `[]` must be an integer OR `*`
meaning all array objects.

```
[example][local][get-data] > extract body samples[*].id --to-var sample-id
[example][local][get-data] > info

      Name:  get-data
    Method:  GET
       URL:  http://{host}/data.json
   Headers:
     Body?:  false

  Input Options
  +------+----------------+
  | name | current values |
  +------+----------------+
  | host | localhost:8000 |
  +------+----------------+

  Output Options
  +-----------------+------+---------------+
  | output variable | type | source        |
  +-----------------+------+---------------+
  | sample-id       | body | samples[*].id |
  +-----------------+------+---------------+

  Planned Requests
  +----------+--------+---------------------------------+---------+-------+
  | name     | method | url                             | headers | body? |
  +----------+--------+---------------------------------+---------+-------+
  | get-data | GET    | http://localhost:8000/data.json |         | false |
  +----------+--------+---------------------------------+---------+-------+

```

Now when we run the request, the `sample-id` variable will be
populated with the response values.

```
[example][local][get-data] > show variables

  +----+-----------+-------------+----------------+----------+
  | id | name      | environment | value          | source   |
  +----+-----------+-------------+----------------+----------+
  | 1  | host      | local       | localhost:8000 | user     |
  | 3  | sample-id | local       | id-1           | get-data |
  | 4  | sample-id | local       | id-2           | get-data |
  +----+-----------+-------------+----------------+----------+

```

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
| :white_check_mark: | tab completion                                   |
| :white_check_mark: | extract from all items in an array               |
| :white_check_mark: | send multiple requests for multiple input opts   |
|                    | extract from other data formats                  |
|                    | option to hide variable values                   |
| :soon:             | run flags                                        |
|                    | run flag for each input option                   |
|                    | clipboard integration                            |
|                    | create request from curl command                 |
|                    | save responses                                   |
|                    | search command                                   |
| :question:         | variable generation                              |
| :question:         | dependency graph                                 |
|                    | global environment                               |
| :white_check_mark: | color requests that have all options satisfied   |
|                    | request statistics                               |
|                    | compare response against previous                |
| :question:         | support for automatic functional testing         |

* :white_check_mark: - In master
* :soon: - In progress
* :question: - Might not happen
* Blank - Rough idea
