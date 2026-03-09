<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import ContextMenu from 'primevue/contextmenu';
import NewJobDialog from '../components/NewJobDialog.vue';
import DownloadsList from '../components/DownloadsList.vue';
import DownloadsToolbar from '../components/DownloadsToolbar.vue';

// Composables
import { useJobs } from '../composables/useJobs';
import { useFileCheck } from '../composables/useFileCheck';
import { useJobMenu } from '../composables/useJobMenu';

// 1. Initialize Logic Engines
const { 
    videos, loading, selectedJobs, 
    canBatchResume, canBatchPause, canBatchDelete, 
    loadVideos, handleBatchResume, handleBatchPause, handleBatchDelete, handleDeleteCompleted 
} = useJobs();

const { missingFileIds, checkMissingFiles, verifyAndOpenFolder } = useFileCheck();

const { cm, menuModel, onRowContextMenu } = useJobMenu(selectedJobs, {
    resume: handleBatchResume,
    pause: handleBatchPause,
    delete: handleBatchDelete,
    openFolder: verifyAndOpenFolder
});

// 2. Local State
const showNewJobDialog = ref(false);
let pollInterval: ReturnType<typeof setInterval> | null = null;

// 3. Lifecycle (Start the Engine)
onMounted(async () => {
    await loadVideos();
    checkMissingFiles(videos.value);
    pollInterval = setInterval(loadVideos, 1000);
});

onUnmounted(() => {
    if (pollInterval) clearInterval(pollInterval);
});

const handleJobStarted = () => {
    // showNewJobDialog.value = false;
    loadVideos();
};
</script>

<template>
    <div class="container">
        <ContextMenu ref="cm" :model="menuModel" />

        <DownloadsToolbar 
            :canResume="canBatchResume"
            :canPause="canBatchPause"
            :canDelete="canBatchDelete"
            @new-job="showNewJobDialog = true"
            @resume="handleBatchResume"
            @pause="handleBatchPause"
            @delete="handleBatchDelete"
            @clear-completed="handleDeleteCompleted"
        />

        <DownloadsList
            :videos="videos"
            :loading="loading"
            v-model:selection="selectedJobs"
            :missingFileIds="missingFileIds"
            @open-folder="verifyAndOpenFolder"
            @row-context-menu="onRowContextMenu" 
        />

        <NewJobDialog
            v-model:visible="showNewJobDialog"
            @job-started="handleJobStarted"
        />
    </div>
</template>

<style scoped>
.container { 
    padding: 1rem; 
    height: 100%;
    display: flex;
    flex-direction: column;
}
</style>