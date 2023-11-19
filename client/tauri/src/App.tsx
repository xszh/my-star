import { ServiceManager, useService } from "./services";

import { Chat } from "./components/chat";
import { Tab, Tabs } from "./components/tab";

import { useEffect } from "react";

import { AudioTab } from "./components/audio-tab";
import { Test } from "./components/test";

import { observer } from "mobx-react";

import "./App.less";

export const App = observer(function App() {
  const auth = useService().get("auth");
  useEffect(() => {
    ServiceManager.init();
    return () => {
      ServiceManager.destroy();
    };
  }, []);
  return (
    <div className="app">
      <Chat />
      <Tabs defaultKey="test">
        <Tab id="audio" title="Audio">
          <AudioTab />
        </Tab>
        <Tab id="info" title="Info">
          <div>{`Token: ${auth.token}`}</div>
        </Tab>
        <Tab id="test" title="Test">
          <Test />
        </Tab>
      </Tabs>
    </div>
  );
});
