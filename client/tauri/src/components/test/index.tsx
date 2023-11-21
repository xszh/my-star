import { observer } from "mobx-react";
import { Button } from "../button";
import { useService } from "@services";

export const Test: React.FC = observer(function Test() {
  const { connect, disconnect, connected } = useService().get("ws");
  return (
    <div>
      <div>Test</div>
      <Button
        onClick={() => {
          try {
            connected ? disconnect() : connect();
          } catch (error) {
            console.error(error);
          }
        }}
      >
        {`${connected ? 'Disconnect' : 'Connect'} WS`}
      </Button>
    </div>
  );
});
