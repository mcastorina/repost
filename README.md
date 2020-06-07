# repost
`repost` is a tool to easily define and send HTTP requests.

## Usage
There are three steps to get started. First you must create a
request, then you define the required variables, then you can
send it any number of times.

```
# create a request
# repost create request <name> <url> [-m method] [-H header ...] [-d body]
#   method is inferred from name or can be explicitly defined via -m
#   - get           GET
#   - post, create  POST
#   - delete        DELETE
#   - put, replace  PUT
#   - patch, update PATCH

repost create request get_health '{host}/v1/health'

# create variables
# repost create variable [-s|-r] <name> <environment>=<value> [environment=value ...]
#   default constant variable
#   -s for script generator (value is script to run)
#   -r for request generator (value is request_name:json_path)

repost create variable host local=localhost:8080

# run the request
# repost <environment> <request_name> [request_name ...]

repost local get_health
```

## Design
There are two main resources managed in `repost`: **requests** and **variables**.
All resources are stored in `$HOME/.config/repost/`.

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
