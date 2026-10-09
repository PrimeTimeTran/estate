type RGB = {
  r: number;
  g: number;
  b: number;
};

type KeyboardKeyEvent = {
  type: "key_down" | "key_up";
  key: string;
};

const keyboardPanel = document.querySelector<HTMLElement>("#keyboard-panel");

if (!keyboardPanel) {
  throw new Error("Keyboard panel was not found");
}

const panel = keyboardPanel;
const select = panel.querySelector<HTMLSelectElement>("select");

if (!select) {
  throw new Error("Keyboard theme selector was not found");
}

const keys = panel.querySelectorAll<HTMLElement>(".key");

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
} satisfies Record<string, RGB[]>;

type ThemeName = keyof typeof themes;

function getRandomInt(min: number, max: number): number {
  const minInt = Math.ceil(min);
  const maxInt = Math.floor(max);

  return Math.floor(Math.random() * (maxInt - minInt) + minInt);
}

function isThemeName(name: string): name is ThemeName {
  return Object.prototype.hasOwnProperty.call(themes, name);
}

function setRandomKeyColor(key: HTMLElement, color: RGB): void {
  const { r, g, b } = color;

  key.style.setProperty("--color", `rgb(${r},${g},${b})`);
  key.style.setProperty("--box-shadow", `rgba(${r},${g},${b},0.5)`);
  key.style.setProperty("--box-shadow-inner", `rgba(${r},${g},${b},0.05)`);
  key.style.setProperty("--text-shadow", `rgba(${r},${g},${b},0.25)`);
}

function setTheme(name: ThemeName): void {
  const theme = themes[name];

  keys.forEach((key) => {
    setRandomKeyColor(key, theme[getRandomInt(0, theme.length)]);
  });
}

select.addEventListener("change", () => {
  if (isThemeName(select.value)) {
    setTheme(select.value);
  }
});

if (isThemeName(select.value)) {
  setTheme(select.value);
}

window.addEventListener("estate-key-event", (event: Event) => {
  handleNativeEvent((event as CustomEvent<KeyboardKeyEvent>).detail);
});

function handleNativeEvent(event: KeyboardKeyEvent): void {
  switch (event.type) {
    case "key_down":
      pressKey(event.key);
      break;
    case "key_up":
      releaseKey(event.key);
      break;
  }
}

function findKey(keyName: string): HTMLElement | null {
  return panel.querySelector(`.key[data-key="${CSS.escape(keyName)}"]`);
}

function pressKey(keyName: string): void {
  const key = findKey(keyName);

  if (!key) {
    console.warn("Unknown key:", keyName);
    return;
  }

  key.classList.add("pressed");
}

function releaseKey(keyName: string): void {
  findKey(keyName)?.classList.remove("pressed");
}

const testKeys = Array.from(keys);

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function randomKeyboardTest(): Promise<void> {
  for (let index = 0; index < 100; index++) {
    const key = testKeys[getRandomInt(0, testKeys.length)];
    const keyName = key.dataset.key;

    if (!keyName) {
      continue;
    }

    pressKey(keyName);
    await sleep(getRandomInt(50, 180));
    releaseKey(keyName);
    await sleep(getRandomInt(30, 250));
  }
}
