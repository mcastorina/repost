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
This section will walk you through creating a request, defining
variables, and adding extractors.

```
repost
[repost] > show requests
[*] No data returned.
[repost] > create request get-example {host}/api/example
[repost] > create variable host local=localhost:8080 stage=https://stage.example.com
[repost] > use environment stage
[repost][stage] > use get-example
[repost][stage][get-example] > info

      Name: get-example
    Method: GET
       URL: {host}/api/example
   Headers:
      Body:

Description:
  None

Options:
  Name      Current Value              Required  Description
  --------  -------------              --------  -----------
  HOST      https://stage.example.com  yes       none

[repost][stage][get-example] > use environment stage
[repost][local][get-example] > options show

Options:
  Name      Current Value          Required  Description
  --------  -------------          --------  -----------
  HOST      localhost:8080         yes       none

[repost][local][get-example] > run
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
