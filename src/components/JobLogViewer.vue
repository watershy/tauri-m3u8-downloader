<script setup lang="ts">
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';
import { ref, nextTick, watch } from 'vue';
import { useJobLogs } from '../composables/useJobLogs';

const props = defineProps<{
    jobId: string;
}>();

// 1. Init Logic Engine
const { logs: generalLogs } = useJobLogs(props.jobId, 'general');
const { logs: downloadLogs } = useJobLogs(props.jobId, 'download');
const { logs: mergeLogs } = useJobLogs(props.jobId, 'merge');

// 2. UI State
const activeTabValue = ref("0");
const scrollContainers = ref<HTMLDivElement[]>([]);
const isUserAtBottom = ref([true, true, true]);

const setContainerRef = (el: any, index: number) => {
    if (el) {
        scrollContainers.value[index] = el as HTMLDivElement;
    }
};

// 3. Auto-Scroll Logic
// We watch the 'logs' data. Whenever it updates, we decide if we scroll.
watch(generalLogs, () => { if (isUserAtBottom.value[0]) scrollToBottom(0); });
watch(downloadLogs, () => { if (isUserAtBottom.value[1]) scrollToBottom(1); });
watch(mergeLogs, () => { if (isUserAtBottom.value[2]) scrollToBottom(2); });

const scrollToBottom = async (tabIndex: number) => {
await nextTick();
    const container = scrollContainers.value[tabIndex];
    if (container) {
        container.scrollTop = container.scrollHeight;
        isUserAtBottom.value[tabIndex] = true;
    }
};

const handleScroll = (event: Event, tabIndex: number) => {
    const el = event.target as HTMLDivElement;
    // Tolerance of 50px to detect if user is near bottom
    const distanceToBottom = el.scrollHeight - (el.scrollTop + el.clientHeight);
    isUserAtBottom.value[tabIndex] = distanceToBottom < 50;
};
</script>

<template>
    <div class="log-wrapper">
        <div class="log-toolbar">
            <span><i class="pi pi-terminal"></i> Job Logs</span>
            <div class="toolbar-right">
                <button 
                    v-if="!isUserAtBottom[Number(activeTabValue)]" 
                    class="jump-button" 
                    @click="scrollToBottom(Number(activeTabValue))"
                >
                    <i class="pi pi-arrow-down"></i> Jump to Bottom
                </button>
                <span class="job-id">{{ jobId }}</span>
            </div>
        </div>
        
        <Tabs v-model:value="activeTabValue">
            <TabList>
                <Tab value="0">General</Tab>
                <Tab value="1">Download</Tab>
                <Tab value="2">Merge</Tab>
            </TabList>
            
            <TabPanels>
                <TabPanel value="0">
                    <div :ref="(el) => setContainerRef(el, 0)" class="log-console" @scroll="handleScroll($event, 0)">
                        <pre v-html="generalLogs"></pre>
                    </div>
                </TabPanel>

                <TabPanel value="1">
                    <div :ref="(el) => setContainerRef(el, 1)" class="log-console" @scroll="handleScroll($event, 1)">
                        <pre v-html="downloadLogs"></pre>
                    </div>
                </TabPanel>

                <TabPanel value="2">
                    <div :ref="(el) => setContainerRef(el, 2)" class="log-console" @scroll="handleScroll($event, 2)">
                        <pre v-html="mergeLogs"></pre>
                    </div>
                </TabPanel>
            </TabPanels>
        </Tabs>
    </div>
</template>

<style scoped>
.log-wrapper {
    background-color: #1e1e1e;
    border-radius: 6px;
    overflow: hidden;
    border: 1px solid #333;
    margin: 5px 0;
}

.log-toolbar {
    background-color: #2d2d2d;
    color: #ccc;
    padding: 5px 10px;
    font-size: 0.8rem;
    border-bottom: 1px solid #333;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.toolbar-right {
    display: flex;
    gap: 10px;
    align-items: center;
}

.jump-button {
    background-color: #3b82f6;
    color: white;
    padding: 2px 8px;
    border: none;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 5px;
    transition: background-color 0.2s;
}

.jump-button:hover {
    background-color: #2563eb;
}

.job-id {
    font-family: monospace;
    opacity: 0.6;
}

.log-console {
    width: 100%;
    height: 300px;
    overflow-y: auto;
    padding: 10px;
    color: #d4d4d4;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
}

.log-console pre {
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    font-family: inherit;
}

/* Scrollbar Styling */
.log-console::-webkit-scrollbar { width: 8px; }
.log-console::-webkit-scrollbar-track { background: #1e1e1e; }
.log-console::-webkit-scrollbar-thumb { background: #444; border-radius: 4px; }
.log-console::-webkit-scrollbar-thumb:hover { background: #555; }

:deep(.p-tabview) {
    background-color: transparent;
}

:deep(.p-tabs) {
    background-color: transparent;
}

/* The row holding the tabs */
:deep(.p-tablist) {
    background-color: #252526;
    border-bottom: 1px solid #333;
}
:deep(.p-tablist-content) {
    background-color: transparent;
}

/* Individual Unselected Tabs */
:deep(.p-tab) {
    background-color: #252526;
    color: #888;
    font-size: 0.75rem;
    padding: 8px 16px;
    border: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    transition: all 0.2s;
}

/* Hover effect for unselected tabs */
:deep(.p-tab:not(.p-tab-active):hover) {
    background-color: #2d2d2d;
    color: #ccc;
}

/* The Active Tab */
:deep(.p-tab-active) {
    background-color: #1e1e1e;
    color: #fff;
    border-bottom: 2px solid #3b82f6;
}

/* The Content Area */
:deep(.p-tabpanels) {
    background-color: #1e1e1e;
    padding: 0;
    border: none;
}
:deep(.p-tabpanel) {
    padding: 0;
}

:deep(.log-info) { color: #3b82f6; }
:deep(.log-debug) { color: #888888; }
:deep(.log-warn) { color: #eab308; }
:deep(.log-error) { 
    color: #ef4444;
    font-weight: bold; 
}
:deep(.log-tag) { color: #a855f7; }
</style>