function getRandomInt(min, max) {
  const minInt = Math.ceil(min);
  const maxInt = Math.floor(max);

  return Math.floor(Math.random() * (maxInt - minInt) + minInt);
}

const select = document.querySelector("select");
const keys = document.querySelectorAll(".key");

const themes = {
  spring: [
    { r: 174, g: 195, b: 214 },
    { r: 135, g: 172, b: 139 },
    { r: 204, g: 213, b: 132 },
    { r: 175, g: 121, b: 219 },
    { r: 92, g: 140, b: 154 },
  ],

  summer: [
    { r: 0, g: 107, b: 166 },
    { r: 4, g: 150, b: 255 },
    { r: 255, g: 188, b: 66 },
    { r: 216, g: 17, b: 89 },
    { r: 143, g: 45, b: 86 },
  ],

  autumn: [
    { r: 96, g: 108, b: 56 },
    { r: 140, g: 159, b: 104 },
    { r: 214, g: 210, b: 184 },
    { r: 221, g: 161, b: 94 },
    { r: 188, g: 108, b: 37 },
  ],

  winter: [
    { r: 89, g: 187, b: 255 },
    { r: 190, g: 233, b: 232 },
    { r: 98, g: 182, b: 203 },
    { r: 202, g: 233, b: 255 },
    { r: 95, g: 168, b: 211 },
  ],

  custom: [
    { r: 0, g: 83, b: 255 },
    { r: 0, g: 239, b: 255 },
    { r: 0, g: 255, b: 135 },
    { r: 70, g: 191, b: 176 },
  ],
};

function setRandomKeyColor(key, color) {
  const { r, g, b } = color;

  key.style.setProperty("--color", `rgb(${r},${g},${b})`);
  key.style.setProperty("--box-shadow", `rgba(${r},${g},${b},0.5)`);
  key.style.setProperty("--box-shadow-inner", `rgba(${r},${g},${b},0.05)`);
  key.style.setProperty("--text-shadow", `rgba(${r},${g},${b},0.25)`);
}

function setTheme(name) {
  const theme = themes[name];

  keys.forEach((key) => {
    setRandomKeyColor(key, theme[getRandomInt(0, theme.length)]);
  });
}

select.addEventListener("change", (event) => {
  setTheme(event.target.value);
});

setTheme(select.value);

// ============================================================
// Estate native event bridge
// ============================================================

const socket = new WebSocket("ws://127.0.0.1:50052");

socket.addEventListener("open", () => {
  console.log("Estate WebSocket connected");
});

socket.addEventListener("close", () => {
  console.log("Estate WebSocket disconnected");
});

socket.addEventListener("error", (error) => {
  console.error("Estate WebSocket error:", error);
});

socket.addEventListener("message", (message) => {
  let event;
  try {
    event = JSON.parse(message.data);
  } catch (error) {
    console.error("Invalid Estate event:", message.data);
    return;
  }
  handleNativeEvent(event);
});

function handleNativeEvent(event) {
  switch (event.type) {
    case "key_down":
      pressKey(event.key);
      break;
    case "key_up":
      releaseKey(event.key);
      break;
    default:
      break;
  }
}

function findKey(keyName) {
  return document.querySelector(`.key[data-key="${CSS.escape(keyName)}"]`);
}

function pressKey(keyName) {
  const key = findKey(keyName);

  if (!key) {
    console.warn("Unknown key:", keyName);
    return;
  }

  key.classList.add("pressed");
}

function releaseKey(keyName) {
  const key = findKey(keyName);

  if (!key) {
    return;
  }

  key.classList.remove("pressed");
}

// ============================================================
// Random keyboard test
// ============================================================

const testKeys = [...keys];

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function randomKeyboardTest() {
  for (let i = 0; i < 100; i++) {
    const key = testKeys[getRandomInt(0, testKeys.length)];
    const keyName = key.dataset.key;
    pressKey(keyName);
    // Hold the key briefly.
    await sleep(getRandomInt(50, 180));
    releaseKey(keyName);
    // Random pause before next key.
    await sleep(getRandomInt(30, 250));
  }
}

randomKeyboardTest();
