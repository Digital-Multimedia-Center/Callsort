<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';

const inputPaths = ref([]);
const outputPath = ref("");
const columnIndex = ref(0);
const outputFormat = ref("csv");
const message = ref("");
const tablePreviews = ref([]); // Holds previews of tables

// Pick multiple CSV/XLSX files
async function selectInputFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: "CSV/Excel", extensions: ["csv", "xls", "xlsx"] }]
  });
  if (!selected) return;
  inputPaths.value = Array.isArray(selected) ? selected : [selected];

  try {
    await invoke("read_input", { input_paths: inputPaths.value });
    message.value = `Loaded ${inputPaths.value.length} file(s)`;
  } catch (err) {
    message.value = `Error loading files: ${err}`;
  }
}

// Pick output directory
async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) outputPath.value = selected;
}

// Sort files using Rust backend
async function sortFiles() {
  if (!inputPaths.value.length || !outputPath.value) {
    message.value = "Select input files and output directory first.";
    return;
  }

  const columnIndices = Array(inputPaths.value.length).fill(Number(columnIndex.value));
  const outputPaths = inputPaths.value.map(path => {
    const filename = path.split("/").pop().split(".")[0];
    return `${outputPath.value}/${filename}_sorted.${outputFormat.value}`;
  });

  try {
    const result = await invoke("sort_file", {
      column_indices: columnIndices,
      output_paths: outputPaths,
      output_format: outputFormat.value
    });
    message.value = result;
  } catch (err) {
    message.value = `Error: ${err}`;
  }
}

// Preview tables using Rust backend
async function previewTables() {
  try {
    const previews = await invoke("preview_tables");
    tablePreviews.value = previews;
  } catch (err) {
    message.value = `Error fetching previews: ${err}`;
  }
}
</script>

<template>
  <div class="app">
    <h1>CallSort Test</h1>

    <div class="section">
      <button @click="selectInputFiles">Select Input Files</button>
      <p v-if="inputPaths.length">{{ inputPaths.length }} file(s) selected</p>
    </div>

    <div class="section">
      <button @click="selectOutputDir">Select Output Directory</button>
      <p v-if="outputPath">{{ outputPath }}</p>
    </div>

    <div class="section">
      <label>Column Index (0-based)</label>
      <input type="number" v-model="columnIndex" min="0" />
    </div>

    <div class="section">
      <label>Output Format</label>
      <select v-model="outputFormat">
        <option value="csv">CSV</option>
        <option value="xlsx">XLSX</option>
      </select>
    </div>

    <div class="section">
      <button @click="sortFiles">Sort Files</button>
      <button @click="previewTables">Preview Tables</button>
    </div>

    <p v-if="message">{{ message }}</p>

    <div v-if="tablePreviews.length" class="previews">
      <h2>Table Previews</h2>
      <div v-for="(table, index) in tablePreviews" :key="index" class="table-preview">
        <h3>Table {{ index + 1 }}</h3>
        <table border="1">
          <thead>
            <tr>
              <th v-for="(header, i) in table.headers" :key="i">{{ header.Text || header.Int || header.Float || "" }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, rIndex) in table.rows" :key="rIndex">
              <td v-for="(cell, cIndex) in row" :key="cIndex">{{ cell.Text || cell.Int || cell.Float || "" }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<style>
.app { font-family: sans-serif; padding: 2rem; }
.section { margin-bottom: 1rem; }
button { padding: 0.5rem 1rem; margin-right: 1rem; cursor: pointer; }
input, select { padding: 0.3rem 0.5rem; }
.table-preview { margin-top: 1rem; }
table { border-collapse: collapse; width: 100%; margin-bottom: 1rem; }
th, td { padding: 0.3rem 0.5rem; text-align: left; }
</style>

