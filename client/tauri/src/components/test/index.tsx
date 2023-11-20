import { useEffect } from "react";
import { Button } from "../button";

export const Test: React.FC = function Test() {
  return (
    <div>
      <div>Test</div>
      <Button
        onClick={() => {
          try {
            const socket = new WebSocket("wss://www.miemie.tech/mystar/ws/");
            // Connection opened
            socket.addEventListener("open", (event) => {
              socket.send("Hello Server!");
            });

            // Listen for messages
            socket.addEventListener("message", (event) => {
              console.log("Message from server ", event.data);
            });
          } catch (error) {
            console.error(error);
          }
        }}
      >
        Connect WS
      </Button>
    </div>
  );
};
