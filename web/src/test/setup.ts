import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import { __resetPollingClock } from "../pollingClock";

afterEach(() => {
  cleanup();
  __resetPollingClock();
});
