import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App.tsx";
import { observer } from "mobx-react";
import "./index.less";
import './assets/iconfont.js';

const Root = observer(function Root() {
  return (
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
});

ReactDOM.createRoot(document.getElementById("root")!).render(<App />);
