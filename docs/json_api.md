# JSON Data API

## GET `/api/ups`

Returns a collection of UPS information.

- **HTTP 200 (Ok)**: Success. The response body will contain an array of UPS objects.

### Example

```http request
GET /api/ups
Accept: application/json
```

### **HTTP 200** response JSON Schema

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string"
      },
      "desc": {
        "type": "string"
      },
      "vars": {
        "type": "object",
        "additionalProperties": {
          "oneOf": [
            {
              "type": "number"
            },
            {
              "type": "string"
            }
          ]
        }
      },
      "cmds": {
        "type": "array",
        "items": {
          "type": "string"
        }
      }
    }
  }
}
```

## GET `/api/ups/:ups_name`

Returns information about a specific UPS, identified by its name.

- **HTTP 200 (Ok)**: Success
- **HTTP 404 (Not Found)**: The specified UPS name was not found.

### Example

```http request
GET /api/ups/bx1600mi
Accept: application/json
```

### **HTTP 200** response JSON Schema

```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string"
    },
    "desc": {
      "type": "string"
    },
    "vars": {
      "type": "object",
      "additionalProperties": {
        "oneOf": [
          {
            "type": "number"
          },
          {
            "type": "string"
          }
        ]
      }
    },
    "cmds": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
```

## POST `/api/ups/:ups_name/command`

Sends a command to be executed on the specified UPS.

- **HTTP 202 (Accepted)**: The command was accepted and sent to the UPS.
- **HTTP 403 (Unauthorized)**: The user is not authorized to execute the specified
  command on the given UPS.
- **HTTP 404 (Not Found)**: The specified UPS name was not found.

### Request Body

```json
{
  "type": "object",
  "properties": {
    "cmd": {
      "type": "string"
    }
  }
}
```

### Example

```http request
POST /api/ups/bx1600mi/command
Accept: application/json
Content-Type: application/json

{
  "cmd": "beeper.enable"
}
```

### **HTTP 403** response JSON Schema

```json
{
  "type": "object",
  "properties": {
    "message": {
      "type": "string"
    },
    "reason": {
      "type": "string"
    }
  }
}
```