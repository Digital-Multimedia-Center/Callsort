<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { basename, join } from "@tauri-apps/api/path";


const inputPaths = ref([]);
const outputPath = ref("");
const outputFormat = ref("csv");
const message = ref("");
const isLoaded = ref(false);

// Array of table data with their previews and selected columns
const tables = ref([]);

async function selectInputFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: "CSV/Excel", extensions: ["csv", "xls", "xlsx", "xlsb"] }]
  });
  if (selected) {
    inputPaths.value = Array.isArray(selected) ? selected : [selected];
    message.value = `${inputPaths.value.length} file(s) selected. Click "Load Files" to preview.`;
    isLoaded.value = false;
    tables.value = [];
  }
}

async function loadFiles() {
  if (inputPaths.value.length === 0) {
    message.value = "Please select files first.";
    return;
  }

  message.value = "Loading files...";
  
  try {
    await invoke("read_input", {
      input_paths: inputPaths.value
    });

    // Get previews from backend
    const previews = await invoke("preview_tables");
    
    // Initialize tables array with preview data and default column selection
    tables.value = previews.map((preview, index) => ({
      filename: inputPaths.value[index].split("/").pop(),
      fullPath: inputPaths.value[index],
      headers: preview.headers,
      rows: preview.rows,
      selectedColumn: 0 // Default to first column
    }));

    isLoaded.value = true;
    message.value = `Loaded ${tables.value.length} file(s) successfully. Click on column headers to select sort column.`;
  } catch (err) {
    message.value = `Error loading files: ${err}`;
    isLoaded.value = false;
  }
}

async function selectOutputDir() {
  const selected = await open({
    directory: true,
    multiple: false
  });
  if (selected) outputPath.value = selected;
}

function getCellValue(cell) {
  return cell.Text || cell.Int || cell.Float || "";
}

function selectColumn(tableIndex, columnIndex) {
  tables.value[tableIndex].selectedColumn = columnIndex;
}

async function sortFiles() {
  if (!isLoaded.value) {
    message.value = "Please load files first.";
    return;
  }

  if (!outputPath.value) {
    message.value = "Please select output directory.";
    return;
  }

  message.value = "Sorting files...";

  try {
    const columnIndices = tables.value.map(t => t.selectedColumn);
    const outputPaths = await Promise.all(
      tables.value.map(async (t) => {
        const nameOnly = await basename(t.filename); // e.g., "small_input.csv"
        const filename = nameOnly.split(".")[0];     // "small_input"
        return await join(outputPath.value, `${filename}_sorted.${outputFormat.value}`);
      })
    );

    const result = await invoke("sort_file", {
      column_indices: columnIndices,
      output_paths: outputPaths,
      output_format: outputFormat.value
    });

    message.value = result || `Sorted ${tables.value.length} file(s) successfully.`;
  } catch (err) {
    message.value = `Error sorting: ${err}`;
  }
}
</script>

<template>
  <div class="app-wrapper">
    <header class="app-header">
      <h1>CallSort</h1>
    </header>
    
    <main class="container">
      <div class="content-wrapper">
        <!-- File Selection -->
        <div class="inputs">
          <div class="form-group">
            <label>Input Files</label>
            <button @click="selectInputFiles" class="select-btn">
              {{ inputPaths.length ? 'Change Files' : 'Select Files' }}
            </button>
            <p v-if="inputPaths.length" class="file-count">
              {{ inputPaths.length }} file(s) selected
            </p>
          </div>

          <div v-if="inputPaths.length && !isLoaded" class="form-group">
            <button @click="loadFiles" class="load-btn">
              Load Files & Preview
            </button>
          </div>

          <!-- Output Directory -->
          <div v-if="isLoaded" class="form-group">
            <label>Output Directory</label>
            <button @click="selectOutputDir" class="select-btn">
              {{ outputPath ? 'Change Directory' : 'Choose Directory' }}
            </button>
            <p v-if="outputPath" class="path-display" :title="outputPath">
              {{ outputPath }}
            </p>
          </div>

          <!-- Output Format -->
          <div v-if="isLoaded" class="form-group">
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

        <!-- Table Previews -->
        <div v-if="isLoaded" class="previews-section">
          <h2>Click Column Headers to Select Sort Column</h2>
          
          <div v-for="(table, tableIndex) in tables" :key="tableIndex" class="table-card">
            <div class="table-header">
              <h3>{{ table.filename }}</h3>
              <div class="sort-indicator">
                <span class="sort-label">Sorting by:</span>
                <span class="sort-column-name">
                  {{ getCellValue(table.headers[table.selectedColumn]) }}
                </span>
              </div>
            </div>

            <div class="table-preview-wrapper">
              <table class="data-table">
                <thead>
                  <tr>
                    <th 
                      v-for="(header, colIndex) in table.headers" 
                      :key="colIndex"
                      :class="{ 
                        'selected-column': colIndex === table.selectedColumn,
                        'clickable-header': true
                      }"
                      @click="selectColumn(tableIndex, colIndex)"
                      :title="`Click to sort by ${getCellValue(header)}`"
                    >
                      <div class="header-content">
                        <span class="header-text">{{ getCellValue(header) }}</span>
                        <span v-if="colIndex === table.selectedColumn" class="selected-indicator">✓</span>
                      </div>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(row, rowIndex) in table.rows" :key="rowIndex">
                    <td 
                      v-for="(cell, colIndex) in row" 
                      :key="colIndex"
                      :class="{ 'selected-column': colIndex === table.selectedColumn }"
                    >
                      {{ getCellValue(cell) }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Sort Button -->
        <div v-if="isLoaded" class="submit-container">
          <button id="submit" @click="sortFiles">
            Sort All Files
          </button>
          <p v-if="message" class="message" :class="{ error: message.includes('Error') }">
            {{ message }}
          </p>
        </div>
        
        <!-- Message for non-loaded state -->
        <p v-if="!isLoaded && message" class="message" :class="{ error: message.includes('Error') }">
          {{ message }}
        </p>
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
  --card-bg: #252525;
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
  max-width: 1200px;
  margin: 0 auto;
  padding: clamp(0.5rem, 2vh, 1.5rem);
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
  gap: clamp(0.75rem, 2vh, 1rem);
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

.select-btn, .load-btn {
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

.select-btn:hover, .load-btn:hover {
  background: var(--button-hover);
  transform: translateY(-1px);
}

.load-btn {
  background: var(--highlight);
  color: #121212;
  font-size: clamp(1rem, 2.2vw, 1.1rem);
  padding: clamp(0.6rem, 1.8vh, 0.9rem) clamp(1rem, 2.5vw, 1.5rem);
}

.load-btn:hover {
  background: #3ee89f;
}

.file-count {
  margin: 0.25rem 0 0 0;
  font-weight: 600;
  color: var(--highlight);
  font-size: clamp(0.8rem, 2vw, 0.95rem);
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

/* Preview Section */
.previews-section {
  margin-top: 1rem;
}

.previews-section h2 {
  font-size: clamp(1.1rem, 2.5vw, 1.4rem);
  margin: 0 0 1rem 0;
  color: var(--highlight);
}

.table-card {
  background: var(--card-bg);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1.5rem;
  border: 1px solid #333;
}

.table-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  flex-wrap: wrap;
  gap: 1rem;
}

.table-header h3 {
  margin: 0;
  font-size: clamp(1rem, 2.2vw, 1.2rem);
  color: var(--text-primary);
}

.sort-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.8rem;
  background: rgba(75, 255, 179, 0.1);
  border-radius: 6px;
  border: 1px solid var(--highlight);
}

.sort-label {
  font-size: clamp(0.8rem, 1.8vw, 0.9rem);
  color: var(--text-secondary);
}

.sort-column-name {
  font-size: clamp(0.85rem, 1.9vw, 0.95rem);
  font-weight: 600;
  color: var(--highlight);
}

.table-preview-wrapper {
  overflow-x: auto;
  border-radius: 6px;
  border: 1px solid #333;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: clamp(0.75rem, 1.8vw, 0.875rem);
}

.data-table thead {
  background: #1a1a1a;
  position: sticky;
  top: 0;
}

.data-table th,
.data-table td {
  padding: 0.5rem 0.75rem;
  text-align: left;
  border: 1px solid #333;
}

.data-table th {
  font-weight: 600;
  color: var(--text-primary);
}

.data-table th.clickable-header {
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.data-table th.clickable-header:hover {
  background: rgba(57, 108, 216, 0.2);
}

.data-table th.clickable-header:active {
  transform: scale(0.98);
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.header-text {
  flex: 1;
}

.selected-indicator {
  font-size: 1.1em;
  color: var(--highlight);
  font-weight: bold;
  animation: checkmark-pop 0.3s ease;
}

@keyframes checkmark-pop {
  0% {
    transform: scale(0);
  }
  50% {
    transform: scale(1.2);
  }
  100% {
    transform: scale(1);
  }
}

.data-table td {
  color: var(--text-secondary);
}

.data-table .selected-column {
  background: rgba(75, 255, 179, 0.1);
  border-left: 2px solid var(--highlight);
  border-right: 2px solid var(--highlight);
}

.data-table th.selected-column {
  background: rgba(75, 255, 179, 0.2);
  color: var(--highlight);
  font-weight: 700;
}

.data-table tbody tr:hover {
  background: rgba(255, 255, 255, 0.03);
}

/* Submit Container */
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
  height: 8px;
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
@media (max-width: 768px) {
  .table-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .sort-indicator {
    width: 100%;
  }
}

@media (max-width: 400px) {
  .container {
    padding: 0.5rem;
  }
  
  .table-card {
    padding: 0.75rem;
  }
}
</style>
