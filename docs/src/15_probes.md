# Probes

`nut_webgui` has basic probe endpoints to check server health and readiness. The
individual NUT server connections can also be probed by their namespace.

Probes return `text/plain` responses with HTTP codes (200, 404, 500, 503).

## Health

- `/probes/health`
- `/probes/health/<namespace>`


### Responses

|Code|Body    |Description                                    |
|----|--------|-----------------------------------------------|
|200 |OK      |All NUT server connections are operational.    |
|200 |DEGRADED|Some of the NUT server connections are failing.|
|500 |DEAD    |All NUT server connections are failing.        |
|404 |        |Requested namespace does not exist.            |


## Readiness

Initial synchronization between `nut_webgui` and NUT servers can take a few
seconds during startup. Readiness probes can be used to check server status
before redirecting user traffic.

- `/probes/readiness`
- `/probes/readiness/<namespace>`

### Responses

|Code|Body     |Description                                                                                            |
|----|---------|-------------------------------------------------------------------------------------------------------|
|200 |READY    |Some of the NUT server connections are operational, and the server is ready for responses.             |
|503 |NOT-READY|None of the NUT server connections are operational yet, and the server is not ready to serve responses.|
|404 |         |Requested namespace does not exist.                                                                    |
