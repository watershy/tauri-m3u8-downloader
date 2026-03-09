<script setup lang="ts">
import { watch, onMounted } from 'vue';
import { open } from '@tauri-apps/api/dialog';
import { downloadDir } from '@tauri-apps/api/path';

import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Select from 'primevue/select';
import { useConfirm } from "primevue/useconfirm";
import { useToast } from "primevue/usetoast";

// Composables
import { useSettings } from '../composables/useSettings';
import { useNewJob } from '../composables/useNewJob';

// UI Helpers
const confirm = useConfirm();
const toast = useToast();

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
    (e: 'update:visible', value: boolean): void;
    (e: 'job-started'): void;
}>();

// 1. Setup Logic Engines
const { settings, loadSettings } = useSettings();

const { 
    // State
    videoUrl, headers, headerOptions, saveFilename, fileExtension, saveFolder,
    mediaChecked, loadingMedia, loadingDownload, errorMessage, hasResolution, resolutions, selectedResolution, totalSegments,
    extensionOptions,
    // Actions
    resetForm, checkMediaMetadata, validateAndCheckFileStatus, submitJob, addHeader, removeHeader
} = useNewJob();

// 2. Lifecycle & Watchers
onMounted(async () => {
    // Load settings to ensure we have a default path
    await loadSettings();
});

// When dialog opens, reset form and apply default path from Settings
watch(() => props.visible, async (isOpen) => {
    if (isOpen) {
        // Ensure settings are loaded
        if (!settings.value.download_path) await loadSettings();

        // Reset form using the path from settings
        resetForm(settings.value.download_path || await downloadDir());
    }
});

// 3. UI Actions
const closeDialog = () => {
    emit('update:visible', false);
};

const handleSelectFolder = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: saveFolder.value || await downloadDir(),
            title: "Select Download Folder"
        });
        
        if (selected && typeof selected === 'string') {
            saveFolder.value = selected; 
        }
    } catch (err) {
        console.error("Failed to open dialog", err);
    }
};

const handleStartDownload = async () => {
    loadingDownload.value = true;
    try {
        // 1. Validate inputs and check Rust backend for file existence
        const status = await validateAndCheckFileStatus();

        if (status === "Error") return; // errorMessage is already set by composable

        if (status === "Busy") {
            toast.add({ 
                severity: 'error', 
                summary: 'Conflict', 
                detail: 'This file is currently being downloaded. Please choose a different name.', 
                life: 5000 
            });
            return;
        }

        if (status === "Exists") {
            confirm.require({
                message: `The file "${saveFilename.value}.${fileExtension.value}" already exists. Do you want to overwrite it?`,
                header: 'File Exists',
                icon: 'pi pi-exclamation-triangle',
                acceptClass: 'p-button-danger',
                accept: async () => {
                    await executeJobCreation(true);
                }
            });
            return;
        }

        // Status is "Ok" (New file)
        await executeJobCreation(false);
    } finally {
        loadingDownload.value = false; 
    }
};

const executeJobCreation = async (overwrite: boolean) => {
    const success = await submitJob(overwrite);
    if (success) {
        emit('job-started');
        closeDialog();
    }
};
</script>

<template>
    <Dialog 
        :visible="visible" 
        @update:visible="emit('update:visible', $event)" 
        modal
        header="Add New Download" 
        :style="{ width: '500px' }">

        <div class="form-container">
            <div class="input-group">
                <label>Video URL:</label>
                <InputText v-model="videoUrl" placeholder="https://..." class="full-width" />
            </div>

            <div class="input-group">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <label>Headers:</label>
                    <Button icon="pi pi-plus" text rounded size="small" @click="addHeader" aria-label="Add Header" />
                </div>
                
                <div v-for="(header, index) in headers" :key="index" class="header-row" style="margin-bottom: 5px;">
                    <Select 
                        v-model="header.name" 
                        :options="headerOptions" 
                        editable 
                        placeholder="Name" 
                        style="width: 35%" 
                    />
                    <InputText 
                        v-model="header.value" 
                        placeholder="Value" 
                        style="width: 55%" 
                    />
                    <Button 
                        icon="pi pi-minus" 
                        severity="danger" 
                        text 
                        rounded 
                        @click="removeHeader(index)" 
                        style="width: 8%" 
                        aria-label="Remove Header"
                    />
                </div>
            </div>

            <div class="input-group" style="margin-top: 15px;">
                <Button 
                    label="Check Media"
                    icon="pi pi-search"
                    :loading="loadingMedia"
                    @click="checkMediaMetadata"
                />
            </div>

            <div v-if="errorMessage" class="error-text">
                {{ errorMessage }}
            </div>

            <div v-if="mediaChecked" class="download-section">
                <div class="info-row" v-if="totalSegments > 0">
                    <span class="info-label"><i class="pi pi-list"></i> Total Segments:</span>
                    <span class="info-value">{{ totalSegments }}</span>
                </div>
                <hr />

                <div class="input-group" v-if="hasResolution">
                    <label>Available Resolutions:</label>
                    <Select v-model="selectedResolution" :options="resolutions" optionLabel="label" placeholder="Select a resolution" class="w-full" />
                </div>

                <div class="input-group">
                    <label>Save to Folder:</label>
                    <div style="display: flex; gap: 5px;">
                        <InputText v-model="saveFolder" readonly style="flex-grow: 1;" />
                        <Button icon="pi pi-folder-open" @click="handleSelectFolder" severity="secondary" aria-label="Select Folder" />
                    </div>
                </div>

                <div class="input-group">
                    <label>File Name:</label>
                    <div style="display: flex; gap: 5px;">
                        <InputText v-model="saveFilename" style="flex-grow: 1;" />
                        <Select 
                            v-model="fileExtension" 
                            :options="extensionOptions" 
                            optionLabel="label" 
                            optionValue="value" 
                            style="width: 110px;">
                        </Select>
                    </div>
                </div>

                <div class="action-buttons">
                    <Button
                        label="Download"
                        icon="pi pi-download"
                        severity="success"
                        :loading="loadingDownload"
                        @click="handleStartDownload">
                    </Button>
                    <Button label="Cancel" icon="pi pi-times" severity="secondary" @click="closeDialog"></Button>
                </div>
            </div>
        </div>
    </Dialog>
</template>

<style scoped>
.form-container { display: flex; flex-direction: column; gap: 10px; }
.input-group { display: flex; flex-direction: column; gap: 5px; }
.input-group label { font-weight: bold; font-size: 0.85rem; color: #555; }
.header-row { display: flex; justify-content: space-between; }
.download-section { margin-top: 10px; background-color: #f8f9fa; padding: 15px; border-radius: 6px; border: 1px solid #e9ecef; display: flex; flex-direction: column; gap: 12px; }
.action-buttons { display: flex; gap: 10px; justify-content: flex-end; margin-top: 5px; }
.full-width { width: 100%; }
.error-text { color: #dc3545; background-color: #ffe6e6; padding: 8px; border-radius: 4px; font-size: 0.9rem; margin-top: 5px; }
.info-row { display: flex; align-items: center; gap: 10px; font-size: 0.9rem; color: #333; margin-bottom: 5px; }
.info-label { font-weight: bold; color: #555; display: flex; align-items: center; gap: 5px; }
.info-value { background: #e9ecef; padding: 2px 8px; border-radius: 12px; font-weight: 600; color: #007bff; }
</style>