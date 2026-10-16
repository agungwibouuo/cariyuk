import React from "react";
import ReactDOM from "react-dom/client";
import Search from "./windows/Search";
import Settings from "./windows/Settings";
import "./styles.css";

function renderWindow() {
  const hash = window.location.hash.replace("#", "");
  switch (hash) {
    case "settings":
      return <Settings />;
    case "search":
    default:
      return <Search />;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{renderWindow()}</React.StrictMode>,
);
