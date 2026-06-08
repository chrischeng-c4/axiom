/** @jsxRuntime classic */
/** @jsx createElement */
import { createElement, render } from "./mini-react";
import { App } from "./app";
import "./style.css";

const root = document.getElementById("root")!;
render(() => createElement(App, null), root);
