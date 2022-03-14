const net = require("net");
const crypto = require("crypto");
const { escape } = require("querystring");

const hash = crypto.createHash("sha1");
let httpPort = 3000;
let wsPort = 3001;

// Simple HTTP server responds with a simple WebSocket client test
const httpServer = net.createServer((connection) => {
  connection.on("data", () => {
    let content = `<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
  </head>
  <body>
    WebSocket test page
    <script>
      let ws = new WebSocket('ws://localhost:3001');
      ws.onmessage = event => alert('Message from server: ' + event.data);
      ws.onopen = () => ws.send('hello');
    </script>
  </body>
</html>
`;
    connection.write(
      "HTTP/1.1 200 OK\r\nContent-Length: " +
        content.length +
        "\r\n\r\n" +
        content
    );
  });
});
httpServer.listen(httpPort, () => {
  console.log("HTTP server listening on port 3000");
});

// Incomplete WebSocket server
const wsServer = net.createServer((connection) => {
  console.log("Client connected");

  connection.on("data", (data) => {
    map = parseWSRequest(data);
    if (map === undefined) {
      console.error("Invalid headers");
      return;
    }
    createReply(map);
  });

  //connection.write();

  connection.on("end", () => {
    console.log("Client disconnected");
  });
});

wsServer.on("error", (error) => {
  console.error("Error: ", error);
});

wsServer.listen(wsPort, () => {
  console.log("WebSocket server listening on port 3001");
});

const parseWSRequest = (data) => {
  const req = data.toString();
  console.log("Data received from client: ", req);

  let headerSplit = new RegExp("[\r\n]");
  let headers = req.split(headerSplit);
  let headerName;
  let headerValue;
  let map = {};

  headers.forEach((element) => {
    headerName = element.substring(0, element.indexOf(":"));
    headerValue = element.substring(element.indexOf(":") + 2);
    map[headerName] = headerValue;
    console.log(escape(headerName));
  });

  if (!checkHost(map["Host"])) {
    console.error("Host fail");
    return undefined;
  }
  if (!checkUpgrade(map["Upgrade"])) {
    console.error("Upgrade fail");
    return undefined;
  }
  if (!checkConnection(map["Connection"])) {
    console.error("Connection fail");
    return undefined;
  }
  if (!checkSecWebSocketVersion(map["Sec-WebSocket-Version"])) {
    console.error("Version fail");
    return undefined;
  }
  if (!checkOrigin(map["Origin"])) {
    console.error("Origin fail");
    return undefined;
  }

  return map;
};

// const checkRequest = (value) => {
//   return true;
// };

const checkHost = (value) => {
  return value === "localhost:" + wsPort;
};

const checkUpgrade = (value) => {
  return value === "websocket";
};

const checkConnection = (value) => {
  return value.includes("Upgrade");
};

// const checkSecWebSocketKey = (value) => {
//   return value;
// };

const checkSecWebSocketVersion = (value) => {
  return value === "13";
};

const checkOrigin = (value) => {
  return value === "http://localhost:" + httpPort;
};

const createReply = (map) => {
  statusLine = "HTTP/1.1 101 Switching Protocols";
  upgrade = "websocket";
  connection = "Upgrade";
  webAccept = hash
    .update(
      map["Sec-WebSocket-Key"] + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
      "base64"
    )
    .digest("base64");
};
