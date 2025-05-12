/* @refresh reload */
import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import App from "./App";
import Songs from "./Songs";

render(() => (
    <Router>
        <Route path="/" component={App} />
        <Route path="/songs" component={Songs} />
    </Router>
), document.getElementById("root") as HTMLElement);
