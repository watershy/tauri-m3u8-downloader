<script setup lang="ts">
import { onMounted } from 'vue';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';

// Import the brain
import { useSettings } from '../composables/useSettings';

const { settings, loadSettings, browseDownloadFolder, saveSettings } = useSettings();

// Load on mount
onMounted(() => {
    loadSettings();
});
</script>

<template>
    <div class="settings-wrapper">
        <Tabs value="0">
            <div class="settings-layout">
                <TabList class="sidebar">
                    <Tab value="0" class="sidebar-item">
                        <i class="pi pi-cog"></i> General
                    </Tab>
                </TabList>
                <TabPanels class="content">
                    <TabPanel value="0">
                        <div class="form-group">
                            <label>Download Path:</label>
                            <div class="input-group">
                                <input type="text" v-model="settings.download_path" readonly />
                                <button @click="browseDownloadFolder">Browse</button>
                            </div>
                        </div>

                        <div class="form-group" style="margin-top: 20px;">
                            <label>Notifications:</label>
                            <div class="checkbox-group">
                                <input 
                                    type="checkbox" 
                                    id="soundToggle" 
                                    v-model="settings.play_completion_sound" 
                                    @change="saveSettings" 
                                />
                                <label for="soundToggle" style="display:inline; margin-left:8px; font-weight:normal; margin-bottom: 0;">
                                    Play sound when download finishes
                                </label>
                            </div>
                        </div>
                    </TabPanel>
                </TabPanels>
            </div>
        </Tabs>
    </div>
</template>

<style scoped>
.settings-layout {
    display: flex;
    height: 100%;
    min-height: 400px;
}

:deep(.p-tablist.sidebar) {
    flex-direction: column;
    width: 200px;
    border-right: 1px solid #ddd;
    background: #f9f9f9;
}

:deep(.p-tab.sidebar-item) {
    width: 100%;
    border: none;
    border-radius: 0;
    text-align: left;
    padding: 15px 20px;
    color: #555;
    gap: 8px;
}

:deep(.p-tab-active.sidebar-item) {
    background: #e9e9e9;
    font-weight: bold;
    color: #000;
}

:deep(.p-tabpanels.content) {
    flex-grow: 1;
    padding: 30px;
    background: #fff;
}

.form-group label { display: block; margin-bottom: 8px; font-weight: 600; }
.input-group { display: flex; gap: 10px; max-width: 500px; }
.input-group input { flex-grow: 1; padding: 8px; border: 1px solid #ccc; border-radius: 4px; background: #fafafa; }
.input-group button { padding: 8px 16px; cursor: pointer; background: #eee; border: 1px solid #ccc; border-radius: 4px; }
.input-group button:hover { background: #ddd; }

.checkbox-group {
    display: flex;
    align-items: center;
    margin-top: 8px;
}
</style>