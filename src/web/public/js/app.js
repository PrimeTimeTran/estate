console.log("🔥 app.js LOADED");

window.js_test = function (payload) {
  console.log("🔥🔥🔥 JS_TEST CALLED", payload);
  console.log("name:", payload.name);
  console.log("id:", payload.id);
};

window.sendPayloadToRust = (payload) => {
  console.log("[WEB] Sending payload to Rust:", payload);
  if (typeof window.receive_payload !== "function") {
    console.error("[WEB] Rust API not installed!");
    return;
  }
  window.receive_payload(payload);
  console.log("[WEB] Rust receive_payload() returned");
};

window.demoSendPayload = function () {
  window.sendPayloadToRust({
    id: 999,
    name: "FROM JAVASCRIPT",
    active: true,
    count: 42,
    score: 12.34,
    price: 99.99,
    optional: "hello",
    tags: ["javascript", "rust"],
    values: [1, 2, 3],
    meta_hashmap: {
      source: "javascript",
    },
    meta_hashset: [["source", "javascript"]],
    bytes: [1, 2, 3],
    address: {
      street: "JS Street",
      city: "Jacksonville",
      zip: 32202,
    },
    status: "Running",
    children: [],
  });
};
