<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebview } from "@tauri-apps/api/webview";

const inputPath = ref("");
const outputPath = ref("");
const columnName = ref("");
const message = ref("");
const isDragging = ref(false);

async function selectInputFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "CSV", extensions: ["csv"] }]
  });
  if (selected) inputPath.value = selected;
}

async function selectOutputFile() {
  const selected = await open({
    directory: true,
    multiple: false
  });
  if (selected) outputPath.value = selected;
}

async function sortFile() {
  if (!inputPath.value || !outputPath.value || !columnName.value) {
    message.value = "Please provide all fields.";
    return;
  }
  try {
    const result = await invoke("sort_csv", {
      args: {
        input_path: inputPath.value,
        column_name: columnName.value,
        output_path: outputPath.value
      }
    });
    message.value = result;
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
      const paths = event.payload.paths;
      if (paths.length > 0) {
        inputPath.value = paths[0];
      }
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
  <main class="container">
    <h1>LCC CSV Sorter</h1>

    <div
      class="drop-zone"
      :class="{ dragging: isDragging }"
      @click="selectInputFile"
    >
      <p v-if="!inputPath">Drag & Drop CSV Here or Click to Select</p>
      <p v-else>Selected: {{ inputPath }}</p>
    </div>

    <button @click="selectOutputFile">Choose Output Path</button>
    <p>{{ outputPath }}</p>

    <input v-model="columnName" placeholder="Column name (e.g. CallNumber)" />

    <button @click="sortFile">Sort CSV</button>

    <p>{{ message }}</p>
  </main>
</template>


<style scoped>
.container {
  max-width: 600px;
  margin: 5vh auto;
  text-align: center;
  font-family: sans-serif;
}
button {
  padding: 0.5em 1em;
  margin: 10px;
  border-radius: 5px;
  border: none;
  background: #396cd8;
  color: white;
  font-weight: bold;
}
input {
  padding: 0.5em;
  margin: 10px;
  width: 80%;
  border: 1px solid #ccc;
  border-radius: 5px;
}

.drop-zone {
  border: 2px dashed #aaa;
  padding: 2em;
  border-radius: 10px;
  margin: 10px;
  cursor: pointer;
  background-color: #f9f9f9;
  transition: background-color 0.3s ease;
}
.drop-zone.dragging {
  background-color: #dceeff;
  border-color: #396cd8;
}
</style>
