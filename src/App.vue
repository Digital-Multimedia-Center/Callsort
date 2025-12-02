<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebview } from "@tauri-apps/api/webview";

const inputPaths = ref([]);
const outputPath = ref("");
const columnName = ref("");
const outputFormat = ref("csv");
const message = ref("");
const isDragging = ref(false);

async function selectInputFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: "CSV/Excel", extensions: ["csv", "xls", "xlsx", "xlsb"] }]
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
      const fileName = path.split("/").pop().split(".").slice(0, -1).join(".");
      const outputFile = `${outputPath.value}/${fileName}_sorted.${outputFormat.value}`;

      await invoke("sort_file", {
          input_path: path,
          column_name: columnName.value,
          output_path: outputFile,
          output_format: outputFormat.value
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
      <div class="content-wrapper">
        <div class="inputs">
          <div
            class="drop-zone"
            :class="{ dragging: isDragging }"
            @click="selectInputFiles"
          >
            <p v-if="inputPaths.length === 0" class="drop-zone-text">
              Drag & Drop Files or Click to Select
            </p>
            <div v-else class="file-list">
              <p class="file-count">{{ inputPaths.length }} file(s) selected</p>
              <ul>
                <li v-for="path in inputPaths" :key="path" :title="path">
                  {{ path.split('/').pop() }}
                </li>
              </ul>
            </div>
          </div>

          <div class="form-group">
            <label>Output Directory</label>
            <button @click="selectOutputFile" class="select-btn">
              {{ outputPath ? 'Change Directory' : 'Choose Directory' }}
            </button>
            <p v-if="outputPath" class="path-display" :title="outputPath">
              {{ outputPath }}
            </p>
          </div>

          <div class="form-group">
            <label for="column-input">Column Name</label>
            <input 
              id="column-input"
              v-model="columnName" 
              placeholder="e.g. CallNumber"
              class="text-input"
            />
          </div>

          <div class="form-group">
            <label>Output Format</label>
            <div class="format-buttons">
              <button 
                class="format-btn"
                :class="{ active: outputFormat === 'csv' }"
                @click="outputFormat = 'csv'"
              >
                <span class="format-icon">📄</span>
                <span class="format-label">CSV</span>
              </button>
              <button 
                class="format-btn"
                :class="{ active: outputFormat === 'xlsx' }"
                @click="outputFormat = 'xlsx'"
              >
                <span class="format-icon">📊</span>
                <span class="format-label">XLSX</span>
              </button>
            </div>
          </div>
        </div>

        <div class="submit-container">
          <button id="submit" @click="sortFiles">
            Sort Files
          </button>
          <p v-if="message" class="message" :class="{ error: message.includes('Error') }">
            {{ message }}
          </p>
        </div>
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
  --footer-bg: #1a1a1a;
  --text-primary: #e0e0e0;
  --text-secondary: #aaa;
  --highlight: #4bffb3;
  --msu-green: #18453b;
  --button-bg: #396cd8;
  --button-hover: #2f54b3;
  --submit-bg: #2b50a1;
  --submit-hover: #1d3a7d;
  --error: #ff4444;
}

* {
  user-select: none;
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  background-color: var(--surface);
  color: var(--text-primary);
  overflow-x: hidden;
}

.app-wrapper {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.app-header {
  background-color: var(--msu-green);
  padding: clamp(0.5rem, 2vh, 1rem) 0;
  text-align: center;
  color: white;
  flex-shrink: 0;
}

.app-header h1 {
  margin: 0;
  font-size: clamp(1.25rem, 4vw, 2rem);
}

.container {
  flex: 1;
  width: 100%;
  max-width: 700px;
  margin: 0 auto;
  padding: clamp(0.5rem, 2vh, 1rem);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  overflow-y: auto;
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: clamp(1rem, 2vh, 1.5rem);
  min-height: 0;
}

.inputs {
  display: flex;
  flex-direction: column;
  gap: clamp(0.75rem, 2vh, 1.25rem);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group > label {
  font-size: clamp(0.875rem, 2vw, 1rem);
  font-weight: 600;
  color: var(--text-primary);
  text-align: left;
}

.drop-zone {
  border: 2px dashed #555;
  padding: clamp(1rem, 3vh, 2rem);
  border-radius: 10px;
  cursor: pointer;
  background-color: #2a2a2a;
  transition: all 0.3s ease;
  color: var(--text-secondary);
  min-height: clamp(80px, 15vh, 120px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.drop-zone:hover {
  border-color: var(--button-bg);
  background-color: #333;
}

.drop-zone.dragging {
  background-color: #1c3f3f;
  border-color: var(--highlight);
  transform: scale(1.02);
}

.drop-zone-text {
  margin: 0;
  font-size: clamp(0.875rem, 2vw, 1rem);
}

.file-list {
  width: 100%;
  max-height: clamp(100px, 20vh, 200px);
  overflow-y: auto;
}

.file-count {
  margin: 0 0 0.5rem 0;
  font-weight: 600;
  color: var(--highlight);
  font-size: clamp(0.75rem, 2vw, 0.9rem);
}

.file-list ul {
  margin: 0;
  padding: 0;
  list-style: none;
  text-align: left;
}

.file-list li {
  padding: 0.4rem 0.6rem;
  margin: 0.25rem 0;
  background: #1a1a1a;
  border-radius: 4px;
  font-size: clamp(0.75rem, 1.8vw, 0.875rem);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.select-btn {
  background: var(--button-bg);
  color: white;
  font-weight: 600;
  border: none;
  border-radius: 6px;
  padding: clamp(0.5rem, 1.5vh, 0.75rem) clamp(0.75rem, 2vw, 1rem);
  cursor: pointer;
  font-size: clamp(0.875rem, 2vw, 1rem);
  transition: all 0.2s ease;
}

.select-btn:hover {
  background: var(--button-hover);
  transform: translateY(-1px);
}

.select-btn:active {
  transform: translateY(0);
}

.path-display {
  margin: 0.25rem 0 0 0;
  padding: 0.5rem;
  background: #2b2b2b;
  border-radius: 4px;
  font-size: clamp(0.7rem, 1.8vw, 0.8rem);
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
}

.text-input {
  background-color: #2b2b2b;
  color: var(--text-primary);
  border: 2px solid #444;
  border-radius: 6px;
  padding: clamp(0.5rem, 1.5vh, 0.75rem);
  font-size: clamp(0.875rem, 2vw, 1rem);
  width: 100%;
  transition: border-color 0.2s ease;
}

.text-input:focus {
  outline: none;
  border-color: var(--button-bg);
}

.format-buttons {
  display: flex;
  gap: clamp(0.75rem, 2vw, 1rem);
  justify-content: flex-start;
}

.format-btn {
  aspect-ratio: 1;
  width: clamp(80px, 15vw, 110px);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  background: #2a2a2a;
  border: 2px solid #444;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  padding: 0.5rem;
}

.format-btn:hover {
  background: #333;
  border-color: var(--button-bg);
  transform: translateY(-2px);
}

.format-btn.active {
  background: var(--button-bg);
  border-color: var(--button-bg);
  box-shadow: 0 4px 12px rgba(57, 108, 216, 0.4);
}

.format-btn.active:hover {
  background: var(--button-hover);
  border-color: var(--button-hover);
}

.format-icon {
  font-size: clamp(1.5rem, 4vw, 2rem);
}

.format-label {
  font-size: clamp(0.875rem, 2vw, 1rem);
  font-weight: 600;
  color: var(--text-primary);
}

.format-btn.active .format-label {
  color: white;
}

.submit-container {
  padding-top: clamp(0.5rem, 2vh, 1rem);
  border-top: 2px solid #333;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

#submit {
  width: 100%;
  max-width: 300px;
  margin: 0 auto;
  height: clamp(45px, 8vh, 55px);
  font-size: clamp(1rem, 2.5vw, 1.2rem);
  background: var(--submit-bg);
  color: white;
  font-weight: bold;
  border: none;
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
}

#submit:hover {
  background-color: var(--submit-hover);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(43, 80, 161, 0.4);
}

#submit:active {
  transform: translateY(0);
}

.message {
  margin: 0;
  padding: 0.5rem;
  font-size: clamp(0.875rem, 2vw, 1rem);
  word-wrap: break-word;
}

.message.error {
  color: var(--error);
}

.app-footer {
  padding: clamp(0.5rem, 1.5vh, 1rem) 0;
  text-align: center;
  font-size: clamp(0.75rem, 1.8vw, 0.9rem);
  background-color: var(--footer-bg);
  color: var(--text-primary);
  flex-shrink: 0;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: #1a1a1a;
}

::-webkit-scrollbar-thumb {
  background: #444;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #555;
}

/* Responsive adjustments */
@media (max-height: 600px) {
  .drop-zone {
    min-height: 60px;
  }
  
  .file-list {
    max-height: 80px;
  }
}

@media (max-width: 400px) {
  .container {
    padding: 0.5rem;
  }
}
</style>
