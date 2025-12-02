// import React, { useEffect, useState } from "react";
// import { listen } from "@tauri-apps/api/event";
// import {
//   greet,
//   screenReadText,
//   getActiveWindowTitle,
//   simulateKeyboardInput,
//   simulateMouseClick,
//   transcribeAudio,
//   generateResponse,
//   getSystemInfo,
// } from "./api";

// function App() {
//   const [log, setLog] = useState<string[]>([]);
//   const [input, setInput] = useState("");
//   const [transcript, setTranscript] = useState<string>("");
//   const [status, setStatus] = useState("");

//   useEffect(() => {
//     const unlisten = listen<string>("recording-status", (event) => {
//       setStatus(event.payload);
//     });

//     return () => {
//       unlisten.then(f => f());
//     };
//   }, []);


//   useEffect(() => {
//     const unlisten = listen("jarvis:hotkey", (event) => {
//       console.log("Hotkey pressed:", event.payload);
//       addLog("Hotkey triggered!");
//     });

//     return () => {
//       unlisten.then((f) => f());
//     };
//   }, []);


//   const addLog = (msg: string) => setLog((prev) => [...prev, msg]);

//   const testAll = async () => {
//     addLog(await greet("Fynn"));
//     addLog(await screenReadText());
//     addLog(await getActiveWindowTitle());
//     addLog(await simulateKeyboardInput("Hello World!"));
//     addLog(await simulateMouseClick());
//     addLog(await transcribeAudio());
//     addLog(await generateResponse("What is Rust?"));
//     addLog(await getSystemInfo());
//   };

//   const record = async () => {
//     console.log("Recording....")
//     const text = await transcribeAudio()
//     setTranscript(text);
//     addLog(transcript)
//     console.log("You said:", text);
//   };

//   return (
//     <div style={{ padding: 20, fontFamily: "sans-serif" }}>
//       <h1>Jarvis Debug Console</h1>

//       <div style={{ marginBottom: 10 }}>
//         <input
//           type="text"
//           placeholder="Send text to LLM..."
//           value={input}
//           onChange={(e) => setInput(e.target.value)}
//         />
//         <button
//           onClick={async () => {
//             if (!input) return;
//             const res = await generateResponse(input);
//             addLog(`LLM: ${res}`);
//           }}
//         >
//           Send
//         </button>
//       </div>

//       <button onClick={testAll} style={{ marginBottom: 10 }}>
//         Test All Plugins
//       </button>
//       <button
//         onClick={async () => {
//           addLog("Listening...");
//           const transcript = await transcribeAudio();
//           addLog(`You said: ${transcript}`);
//           const reply = await generateResponse(transcript);
//           addLog(`Jarvis: ${reply}`);
//         }}
//       >
//         Talk to Jarvis
//       </button>
//       <button
//         onClick={async () => {
//           const res = await simulateKeyboardInput("Hello from Jarvis!");
//           addLog(res);
//         }}
//       >
//         Simulate Keyboard
//       </button>

//       <button
//         onClick={async () => {
//           const res = await simulateMouseClick();
//           addLog(res);
//         }}
//       >
//         Simulate Mouse Click
//       </button>
//       <button onClick={record}>🎤 Record & Transcribe</button>
//       <button onClick={async () => {
//         try {
//           addLog('Listening....')
//           const text = await screenReadText();
//           addLog(`Screen OCR: ${text}`);
//         } catch (err) {
//           addLog(`Error: ${err}`);
//         }
//       }}>
//         Read Screen
//       </button>
//       <div>{status}</div>
//       <div
//         style={{
//           border: "1px solid #ccc",
//           padding: 10,
//           height: 300,
//           overflowY: "scroll",
//           background: "#f7f7f7",
//         }}
//       >
//         {log.map((l, i) => (
//           <div key={i} style={{ marginBottom: 4 }}>
//             {l}
//           </div>
//         ))}
//       </div>
//     </div>
//   );
// }

// export default App;


// src/App.tsx
import { useEffect, useState } from "react";
import { startMicStream } from "./audio";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [transcript, setTranscript] = useState("");
  const [llmResponse, setLlmResponse] = useState("");

  useEffect(() => {
    startMicStream();

    const unlistenTranscript = listen("transcript", (event) => {
      setTranscript((prev) => prev + " " + event.payload);
    });

    const unlistenLLM = listen("llm-response", (event) => {
      setLlmResponse((prev) => prev + event.payload);
    });

    return () => {
      unlistenTranscript.then((f) => f());
      unlistenLLM.then((f) => f());
    };
  }, []);

  return (
    <div style={{ padding: "2rem" }}>
      <h1>Real-Time Voice Assistant</h1>
      <h2>Transcript:</h2>
      <p>{transcript}</p>
      <h2>Assistant:</h2>
      <p>{llmResponse}</p>
    </div>
  );
}

export default App;
