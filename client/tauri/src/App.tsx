import { ServiceManager } from "./services";

import { Chat } from "./components/chat";
import { Tab, Tabs } from "./components/tab";

import { useEffect } from "react";

import { AudioTab } from "./components/audio-tab";
import { observer } from "mobx-react";

import "./App.less";

export const App = observer(function App() {
  useEffect(() => {
    ServiceManager.init();
    return () => {
      ServiceManager.destroy();
    };
  }, []);
  return (
    <div className="app">
      <Chat />
      <Tabs defaultKey="audio">
        <Tab id="audio" title="Audio">
          <AudioTab />
        </Tab>
        <Tab id="test" title="Test">
          {"Test"}
        </Tab>
      </Tabs>
    </div>
  );
});
