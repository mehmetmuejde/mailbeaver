import { describe, expect, it } from "vitest";
import { mailFixture } from "@/shared/lib/test";
import { groupMessages, NO_PROJECT, UNKNOWN_SENDER } from "./group-messages";

const SHOP = { id: 1, name: "shop" };
const shop = mailFixture({ id: 3, from: "shop@example.com", project: SHOP });
const blog = mailFixture({ id: 2, from: "", project: null });
const shopAgain = mailFixture({ id: 1, from: "shop@example.com", project: SHOP });
const messages = [shop, blog, shopAgain];

describe("groupMessages", () => {
  it("keeps one ungrouped list for the plain view", () => {
    expect(groupMessages(messages, "all")).toEqual([{ key: "all", label: null, messages }]);
  });

  it("groups by sender in order of the newest message", () => {
    const groups = groupMessages(messages, "sender");
    expect(groups.map((group) => group.label)).toEqual(["shop@example.com", UNKNOWN_SENDER]);
    expect(groups[0]?.messages.map((message) => message.id)).toEqual([3, 1]);
  });

  it("groups by project with a group for mail without one", () => {
    const groups = groupMessages(messages, "project");
    expect(groups.map((group) => group.label)).toEqual(["shop", NO_PROJECT]);
  });
});
