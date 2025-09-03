# Completed to send the pcd
## Code settings
```cpp
match send_pcd_unreliable(&session, &pcd, chunk_size, 1).await {
```

## Client
```bash
Loaded 456836 points from data/input/Sample01.pcd
[2025-09-03T22:22:42Z INFO  client] Connecting to https://localhost:4443/
[2025-09-03T22:22:42Z INFO  client] Connected
[2025-09-03T22:22:42Z INFO  client] Sending 456836 points in 4154 chunks via unreliable stream
[2025-09-03T22:22:51Z INFO  client] Send time: 8.665725987s
[2025-09-03T22:22:51Z INFO  client] Successfully sent all PCD data
[2025-09-03T22:22:52Z INFO  client] Client shutting down
```
## Server
```bash
[2025-09-03T22:22:51Z DEBUG server] Accepted datagram: 8
[2025-09-03T22:22:51Z DEBUG server] Transmission complete! Total points: 456836, Total chunks: 4154
[2025-09-03T22:22:51Z INFO  server] Reconstructed PCD data: 456836 total points from 4154 chunks
[2025-09-03T22:22:51Z INFO  server] Saved received PCD to data/output/received.pcd
[2025-09-03T22:22:51Z DEBUG quinn::connection] drive; id=0
[2025-09-03T22:22:51Z DEBUG quinn::connection] drive; id=0
[2025-09-03T22:22:51Z DEBUG quinn::connection] drive; id=0
[2025-09-03T22:22:52Z DEBUG quinn::connection] drive; id=0
[2025-09-03T22:22:52Z INFO  server] Closing session: connection error: closed by peer: 0
[2025-09-03T22:22:52Z DEBUG quinn::connection] drive; id=0
```

## Code settings
```cpp
// match send_pcd_unreliable(&session, &pcd, chunk_size, 1).await {
```

## Client
```bash
Loaded 456836 points from data/input/Sample01.pcd
[2025-09-03T23:03:13Z INFO  client] Connecting to https://localhost:4443/
[2025-09-03T23:03:13Z INFO  client] Connected
[2025-09-03T23:03:13Z INFO  client] Sending 456836 points in 4154 chunks via unreliable stream
[2025-09-03T23:03:13Z INFO  client] Send time: 13.297379ms
[2025-09-03T23:03:13Z INFO  client] Successfully sent all PCD data
[2025-09-03T23:03:16Z INFO  client] Sending 456836 points in 4154 chunks via unreliable stream
[2025-09-03T23:03:16Z INFO  client] Send time: 14.520528ms
[2025-09-03T23:03:16Z INFO  client] Successfully sent all PCD data
[2025-09-03T23:03:16Z INFO  client] Client shutting down
```
## Server
```bash
[2025-09-03T22:36:41Z INFO  server] Accepted session!
[2025-09-03T22:36:41Z INFO  server] Transmission complete! Total points: 456836, Total chunks: 4154
[2025-09-03T22:36:41Z INFO  server] Reconstructed PCD data: 456836 total points from 4154 chunks
[2025-09-03T22:36:41Z INFO  server] Saved received PCD to data/output/received.pcd
[2025-09-03T22:36:42Z INFO  server] Closing session: connection error: closed by peer: 0
```