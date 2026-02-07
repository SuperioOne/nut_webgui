# Events API

## Basic example
*“Let's see Paul Allen's code”*

Instead of long and boring wall of text, here's an basic example using Nodejs.

```javascript
#!/bin/node

const API_KEY = "your regular API key generated from nut_webgui";
const socket = new WebSocket("ws://crazy-nut-server/events");

// Listen for messages
socket.addEventListener("message", (event) => {
  let msg = JSON.parse(event.data);

  switch (msg.type) {
    case "WaitingForAuth":
      // You'll only receive this message if authentication is enabled.
      // If you don't send any message, server timeouts after 30 seconds
      // and closes the socket.

      // Sending LOGIN command with API key
      socket.send(`LOGIN:${API_KEY}`);
      break;

    case "AuthOk":
      // Login command with API key is verified by the server.
      console.log("WE'RE SO BACK");
      break;

    case "HandshakeError":
      // Think of the server as a tsundere character. It calls you a baka and closes the
      // socket connection when the login attempt fails. Do not bother sending
      // any other LOGIN messages after this. Simply create a new socket connection
      // and try again with the proper API key.
      console.warn(`Server is mad at you: ${msg.message}`);
      break;

    case "DeviceConnected":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: Device connected! -> ${msg.name}@${msg.namespace}`,
      );
      break;

    case "DeviceRemoved":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: Device is gone -> ${msg.name}@${msg.namespace}`,
      );
      break;

    case "DeviceUpdate":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: Device info updated -> ${msg.name}@${msg.namespace}`,
      );
      break;

    case "DeviceStatus":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: Device status changed -> ${msg.name}@${msg.namespace}, before: ${msg.status_old}, after: ${msg.status_new}`,
      );
      break;

    case "DaemonStatus":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: UPSD status changed -> ${msg.name}@${msg.namespace}, before: ${msg.status_old}, after: ${msg.status_new}`,
      );
      break;

    case "ClientConnect":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: Client connected to -> ${msg.name}@${msg.namespace}, client_ip: ${msg.client_ip}`,
      );
      break;

    case "ClientDisconnect":
      console.log(
        `${new Date(msg.timestamp).toISOString()}: Client disconnected from -> ${msg.name}@${msg.namespace}, client_ip: ${msg.client_ip}`,
      );
      break;

    case "SessionEnded":
      console.log("Session is closed by the server.");
      socket.close();
      break;

    default:
      console.error("unreachable ???");
      break;
  }
});

socket.addEventListener("close", (event) => {
  console.log("IT'S SO OVER");
});
```

*Example Typescript union type for event messages*

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

When authentication is enabled, the Events API requires authentication through a WebSocket connection.
The server will send a `WaitingForAuth` message first. The client must respond with a `LOGIN:APIKEY` command
within 30 seconds. 

If authentication fails, the server will send a `HandshakeError` message and close
the connection. The client should create a new socket connection and try again with the proper API key.

If authentication succeeds, the server will respond with an `AuthOk` message, and it'll start sending event
messages.

Other than login command, connection is unidirectional. If you send any other message through the
socket, server simply closes the socket.

## Message types

All messages are in JSON formatted text. The Events API supports several message types for monitoring UPS devices and system status:

   - **DeviceConnected** - Communication established with an UPS device
   - **DeviceRemoved** - An UPS device has been removed from the system (NUT server no longer lists the UPS)
   - **DeviceUpdate** - Information about a device has been updated
   - **DeviceStatus** - Status of a device has changed, including old and new status values and associated events
   - **DaemonStatus** - Status of the UPSD server has changed (Online, Dead, or Not Ready)
   - **ClientConnect** - A 'monitoring' client has attached to the UPS device
   - **ClientDisconnect** - A 'monitoring' client has detached from the UPS device
   - **HandshakeError** - Authentication failed with error details
   - **SessionEnded** - The session has ended
   - **WaitingForAuth** - Authentication is required
   - **AuthOk** - Authentication successful

