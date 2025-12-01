import React, { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  greet,
  screenReadText,
  getActiveWindowTitle,
  simulateKeyboardInput,
  simulateMouseClick,
  transcribeAudio,
  generateResponse,
  getSystemInfo,
} from "./api";

function App() {
  const [log, setLog] = useState<string[]>([]);
  const [input, setInput] = useState("");

  useEffect(() => {
    const unlisten = listen("jarvis:hotkey", (event) => {
      console.log("Hotkey pressed:", event.payload);
      addLog("Hotkey triggered!");
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);


  const addLog = (msg: string) => setLog((prev) => [...prev, msg]);

  const testAll = async () => {
    addLog(await greet("Fynn"));
    addLog(await screenReadText());
    addLog(await getActiveWindowTitle());
    addLog(await simulateKeyboardInput("Hello World!"));
    addLog(await simulateMouseClick());
    addLog(await transcribeAudio());
    addLog(await generateResponse("What is Rust?"));
    addLog(await getSystemInfo());
  };

  return (
    <div style={{ padding: 20, fontFamily: "sans-serif" }}>
      <h1>Jarvis Debug Console</h1>

      <div style={{ marginBottom: 10 }}>
        <input
          type="text"
          placeholder="Send text to LLM..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
        />
        <button
          onClick={async () => {
            if (!input) return;
            const res = await generateResponse(input);
            addLog(`LLM: ${res}`);
          }}
        >
          Send
        </button>
      </div>

      <button onClick={testAll} style={{ marginBottom: 10 }}>
        Test All Plugins
      </button>
      <button
        onClick={async () => {
          addLog("Listening...");
          const transcript = await transcribeAudio();
          addLog(`You said: ${transcript}`);
          const reply = await generateResponse(transcript);
          addLog(`Jarvis: ${reply}`);
        }}
      >
        Talk to Jarvis
      </button>
      <button
        onClick={async () => {
          const res = await simulateKeyboardInput("Hello from Jarvis!");
          addLog(res);
        }}
      >
        Simulate Keyboard
      </button>

      <button
        onClick={async () => {
          const res = await simulateMouseClick();
          addLog(res);
        }}
      >
        Simulate Mouse Click
      </button>

      <button onClick={async () => {
        try {
          const text = await screenReadText();
          addLog(`Screen OCR: ${text}`);
        } catch (err) {
          addLog(`Error: ${err}`);
        }
      }}>
        Read Screen
      </button>

      <div
        style={{
          border: "1px solid #ccc",
          padding: 10,
          height: 300,
          overflowY: "scroll",
          background: "#f7f7f7",
        }}
      >
        {log.map((l, i) => (
          <div key={i} style={{ marginBottom: 4 }}>
            {l}
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
