<script setup lang="ts">
import { onMounted } from 'vue';
import ConfirmDialog from 'primevue/confirmdialog';
import Toast from 'primevue/toast';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';

// Import Views
import DownloadsTab from './views/DownloadsTab.vue';
import SettingsTab from './views/SettingsTab.vue';
import GlobalStatusBar from './components/GlobalStatusBar.vue';

import { useAppClose } from './composables/useAppClose';
const { initCloseInterceptor } = useAppClose();

onMounted(() => {
    initCloseInterceptor();
});
</script>

<template>
    <ConfirmDialog />
    <Toast />

    <div class="app-layout">
        <Tabs value="0" class="main-tabs">
            <TabList>
                <Tab value="0">Downloads</Tab>
                <Tab value="1">Settings</Tab>
            </TabList>
            <TabPanels>
                <TabPanel value="0">
                    <DownloadsTab />
                </TabPanel>
                <TabPanel value="1">
                    <SettingsTab />
                </TabPanel>
            </TabPanels>
        </Tabs>

        <GlobalStatusBar />
    </div>
</template>

<style scoped>
.app-layout {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

:deep(.main-tabs) {
    flex: 1;                /* Tells Flexbox to take up all remaining space */
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

:deep(.p-tabpanels) {
    flex: 1;
    overflow-y: auto;       /* Makes the content scrollable while the tabs & footer stay fixed */
    padding: 0; 
}
</style>