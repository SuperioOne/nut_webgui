# Events API

## Basic example
*"Let's see Paul Allen's code"*

Instead of a long and boring wall of text, here's a basic example using Node.js.

```javascript
#!/bin/node

const API_KEY = "your regular API key generated from nut_webgui";
const socket = new WebSocket("ws://crazy-nut-server/events");

// Listen for messages
socket.addEventListener("message", (event) => {
  let msg = JSON.parse(event.data);
  let timestamp = msg.timestamp ? new Date(msg.timestamp).toISOString() : "";

  switch (msg.type) {
    case "WaitingForAuth":
      // You'll only receive this message if authentication is enabled.
      // If you don't send any message, the server times out after 30 seconds
      // and closes the socket.

      // Sending LOGIN command with API key
      socket.send(`LOGIN:${API_KEY}`);
      break;

    case "AuthOk":
      // Login command with API key is verified by the server.
      console.log("WE'RE SO BACK");
      break;

    case "HandshakeError":
      // Authentication failed. The server closes the connection immediately.
      // Do not send any other LOGIN messages. Create a new socket connection
      // and try again with the correct API key.
      console.warn(`Server is mad at you: ${msg.message}`);
      break;

    case "DeviceConnected":
      console.log(`${timestamp}: New device! -> ${msg.name}@${msg.namespace}`);
      break;

    case "DeviceRemoved":
      console.log(
        `${timestamp}: Device is gone -> ${msg.name}@${msg.namespace}`,
      );
      break;

    case "DeviceUpdate":
      console.log(
        `${timestamp}: Device info updated -> ${msg.name}@${msg.namespace}`,
      );
      break;

    case "DeviceStatus":
      console.log(
        `${timestamp}: Device status changed -> ${msg.name}@${msg.namespace}, before: ${msg.status_old}, after: ${msg.status_new}`,
      );
      break;

    case "DaemonStatus":
      console.log(
        `${timestamp}: UPSD status changed -> ${msg.namespace}, status: ${msg.status}`,
      );
      break;

    case "ClientConnect":
      console.log(
        `${timestamp}: Client connected to -> ${msg.name}@${msg.namespace}, client_ip: ${msg.client_ip}`,
      );
      break;

    case "ClientDisconnect":
      console.log(
        `${timestamp}: Client disconnected from -> ${msg.name}@${msg.namespace}, client_ip: ${msg.client_ip}`,
      );
      break;

    case "SessionEnded":
      console.log("Session is closed by the server.");
      socket.close();
      break;

    default:
      console.error("?@@ unreachable");
      break;
  }
});

socket.addEventListener("close", (event) => {
  console.log("IT'S SO OVER");
});
```

*Example TypeScript union type for event messages*

```typescript
type DeviceEventName =
  | "AlarmOn"
  | "AlarmOff"
  | "Boosting"
  | "BoostingEnded"
  | "BypassOn"
  | "BypassOff"
  | "Calibrating"
  | "CalibrationCompleted"
  | "Charging"
  | "ChargingEnded"
  | "Discharging"
  | "DischargingEnded"
  | "FSD"
  | "LowBattery"
  | "LowBatteryEnded"
  | "DeviceOff"
  | "DeviceOn"
  | "Online"
  | "OnBattery"
  | "Overloaded"
  | "OverloadEnded"
  | "ReplaceBattery"
  | "ReplaceBatteryEnded"
  | "Testing"
  | "TestCompleted"
  | "Trimming"
  | "TrimmingEnded"
  | "NoCOMM"
  | "COMM";

type NutEventMessage =
  | {
      type: "DeviceRemoved" | "DeviceConnected" | "DeviceUpdate";
      // Device name
      name: string;
      // UPSD server name
      namespace: string;
      // Event time in unix timestamp (milliseconds)
      timestamp: number;
    }
  | {
      type: "DeviceStatus";
      // Device name
      name: string;
      // UPSD server name
      namespace: string;
      // New status text, for example "OB DISCHRG"
      status_new: string;
      // Previous status, for example "OL"
      status_old: string;
      // Device events based on comparison between old and new status.
      events: DeviceEventName[];
      // Event time in unix timestamp (milliseconds)
      timestamp: number;
    }
  | {
      type: "DaemonStatus";
      // UPSD server name
      namespace: string;
      // UPSD server connection status
      status: "Online" | "Dead" | "Not Ready";
      // Event time in unix timestamp (milliseconds)
      timestamp: number;
    }
  | {
      type: "ClientConnect" | "ClientDisconnect";
      // Client IPv4 or IPv6 address
      client_ip: string;
      // Device name
      name: string;
      // UPSD server name
      namespace: string;
      // Event time in unix timestamp (milliseconds)
      timestamp: number;
    }
  | {
      type: "HandshakeError";
      // Error details
      message: string;
    }
  | { type: "SessionEnded" | "WaitingForAuth" | "AuthOk" };
```

## Initializing the connection

When authentication is enabled, the Events API requires authentication through a
WebSocket connection. The server will send a `WaitingForAuth` message first. The
client must respond with a `LOGIN:APIKEY` command within 30 seconds.

If authentication fails, the server will send a `HandshakeError` message and
close the connection. The client should create a new socket connection and try
again with the proper API key.

If authentication succeeds, the server will respond with an `AuthOk` message,
and it'll start sending event messages.

Other than the login command, the connection is unidirectional. If you send any
other message through the socket, the server simply closes the socket.

## Message types

All messages are in JSON-formatted text. The Events API supports several message
types for monitoring UPS devices and system status:

|Message Type        |Description                                                                              |
|--------------------|-----------------------------------------------------------------------------------------|
|**AuthOk**          |Authentication is successful                                                             |
|**ClientConnect**   |A 'monitoring' client[^nut_clients] has attached to the UPS device                       |
|**ClientDisconnect**|A 'monitoring' client[^nut_clients] has detached from the UPS device                     |
|**DaemonStatus**    |Status of the UPSD server has changed (Online, Dead, or Not Ready)                       |
|**DeviceConnected** |Communication established with an UPS device                                             |
|**DeviceRemoved**   |An UPS device has been removed from the system (NUT server no longer lists the UPS)      |
|**DeviceStatus**    |Status of a device has changed, including old and new status values and associated events|
|**DeviceUpdate**    |Information about a device has been updated                                              |
|**HandshakeError**  |Authentication failed with error details                                                 |
|**SessionEnded**    |The session has ended                                                                    |
|**WaitingForAuth**  |Authentication is required                                                               |

[^nut_clients]: [networkupstools - NUT Clients](https://networkupstools.org/docs/man/nut.html#_nut_clients)
