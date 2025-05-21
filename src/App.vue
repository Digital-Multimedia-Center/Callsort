<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebview } from "@tauri-apps/api/webview";

const inputPaths = ref([]);
const outputPath = ref("");
const columnName = ref("");
const message = ref("");
const isDragging = ref(false);

async function selectInputFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: "CSV", extensions: ["csv", "xls", "xlsx", "xlsb"] }]
  });
  if (selected) {
    inputPaths.value = Array.isArray(selected) ? selected : [selected];
  }
}

async function selectOutputFile() {
  const selected = await open({
    directory: true,
    multiple: false
  });
  if (selected) outputPath.value = selected;
}

async function sortFiles() {
  if (inputPaths.value.length === 0 || !outputPath.value || !columnName.value) {
    message.value = "Please provide all fields.";
    return;
  }

  message.value = "Sorting files...";

  try {
    for (const path of inputPaths.value) {
      await invoke("sort_csv", {
        args: {
          input_path: path,
          column_name: columnName.value,
          output_path: outputPath.value
        }
      });
    }
    message.value = `Sorted ${inputPaths.value.length} file(s) successfully.`;
  } catch (err) {
    message.value = "Error: " + err;
  }
}

let unlisten;
onMounted(async () => {
  const webview = await getCurrentWebview();
  unlisten = await webview.onDragDropEvent((event) => {
    if (event.payload.type === 'over') {
      isDragging.value = true;
    } else if (event.payload.type === 'drop') {
      const paths = event.payload.paths || [];
      const filePaths = paths.filter(p => p.endsWith(".csv") || p.endsWith(".xlsx") || p.endsWith(".xls") || p.endsWith(".xlsb"));
      inputPaths.value = filePaths;
      isDragging.value = false;
    } else {
      isDragging.value = false;
    }
  });
});

onBeforeUnmount(() => {
  if (unlisten) unlisten();
});
</script>

<template>
  <div class="app-wrapper">
    <header class="app-header">
      <h1>CallSort</h1>
    </header>
    <main class="container">

      <div class="inputs">
        <div
          class="drop-zone"
          :class="{ dragging: isDragging }"
          @click="selectInputFiles"
        >
          <p v-if="inputPaths.length === 0">Drag & Drop CSV Files or Click to Select</p>
          <ul v-else>
            <li v-for="path in inputPaths" :key="path">{{ path }}</li>
          </ul>
        </div>

        <button @click="selectOutputFile">Choose Output Directory</button>
        <p>{{ outputPath }}</p>

        <input v-model="columnName" placeholder="Column name (e.g. CallNumber)" />
      </div>

      <div class="submit-container">
        <button id="submit" @click="sortFiles">Sort CSVs</button>
        <p>{{ message }}</p>
      </div>

    </main>
    <footer class="app-footer">
      Made with ❤️ at the Digital Multimedia Center
    </footer>
  </div>
</template>



<style>
:root {
  --background: #121212;
  --surface: #1e1e1e;
  --footer-bg: #1a1a1a; /* Slightly lighter or darker footer, tweak as needed */
  --text-primary: #e0e0e0;
  --text-secondary: #aaa;
  --highlight: #4bffb3;
  --msu-green: #18453b;
  --button-bg: #396cd8;
  --button-hover: #2f54b3;
  --submit-bg: #2b50a1;
  --submit-hover: #1d3a7d;
}

* {
  user-select: none;
}

html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  background-color: var(--surface);
  color: var(--text-primary);
}

.app-wrapper {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.container {
  flex: 1;
  max-width: 600px;
  margin: 0 auto;
  text-align: center;
  font-family: sans-serif;
  padding: 3vh 1rem;
}

.app-header {
  background-color: var(--msu-green);
  padding: 2vh;
  color: white;
  width: 100%;
  padding: 1rem 0;
  text-align: center;
}

.app-header h1 {
  margin: 0;
  font-size: 2rem;
}

.inputs {
  padding: 5vh 0 3vh 0;
}

.inputs button {
  background: var(--button-bg);
  color: white;
  font-weight: bold;
  border: none;
  border-radius: 5px;
  margin: 10px;
  padding: 0.5em 1em;
  cursor: pointer;
}

.inputs button:hover {
  background: var(--button-hover);
}

input {
  background-color: #2b2b2b;
  color: var(--text-primary);
  border: 1px solid #444;
  border-radius: 5px;
  padding: 0.5em;
  margin: 10px;
  width: 80%;
}

.drop-zone {
  border: 2px dashed #888;
  padding: 2em;
  border-radius: 10px;
  margin: 10px;
  cursor: pointer;
  background-color: #2a2a2a;
  transition: background-color 0.3s ease;
  color: var(--text-secondary);
}

.drop-zone.dragging {
  background-color: #1c3f3f;
  border-color: var(--highlight);
}

.submit-container {
  padding: 3vh 0;
  border-top: 2px solid #333;
  text-align: center;
}

#submit {
  width: 40%;
  height: 50px;
  font-size: 1.2rem;
  background: var(--submit-bg);
  color: white;
  font-weight: bold;
  border: none;
  border-radius: 5px;
  transition: background-color 0.2s ease;
  cursor: pointer;
}

#submit:hover {
  background-color: var(--submit-hover);
}

.app-footer {
  padding: 2vh;
  text-align: center;
  font-size: 0.9rem;
  background-color: var(--footer-bg);
  padding: 1rem 0;
  text-align: center;
  color: var(--text-color);
}

</style>
