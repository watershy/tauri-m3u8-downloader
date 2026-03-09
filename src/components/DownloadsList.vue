<script setup lang="ts">
import { ref } from 'vue';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import JobLogViewer from './JobLogViewer.vue'; // Assumes same directory
import { useFormatters } from '../composables/useFormatters';

const { getStatusSeverity } = useFormatters();

const props = defineProps<{
    videos: any[];
    loading: boolean;
    selection: any[];       // For v-model
    missingFileIds: Set<string>;
}>();

const emit = defineEmits<{
    (e: 'update:selection', value: any[]): void; // Standard v-model update
    (e: 'open-folder', job: any): void;
    (e: 'row-context-menu', event: any): void;
}>();

const expandedRows = ref([]);

const rowClassGenerator = (data: any) => props.missingFileIds.has(data.id) ? 'file-missing-row' : '';
</script>

<template>
    <DataTable
        :value="videos"
        :loading="loading && videos.length === 0"
        :selection="selection"
        @update:selection="val => $emit('update:selection', val)"
        dataKey="id"
        showGridlines
        size="small"
        tableStyle="min-width: 50rem"
        v-model:expandedRows="expandedRows"
        :rowClass="rowClassGenerator"
        contextMenu
        @rowContextmenu="$emit('row-context-menu', $event)"
    >
        <Column expander style="width: 3rem" />
        <Column selectionMode="multiple" headerStyle="width: 3rem"></Column>
        <Column field="file_name" header="File Name">
             <template #body="slotProps">
                <span :class="{ 'strikethrough-text': missingFileIds.has(slotProps.data.id) }">
                    {{ slotProps.data.file_name }}
                </span>
             </template>
        </Column>
        <template #expansion="slotProps">
            <div class="p-3">
                <JobLogViewer :jobId="slotProps.data.id" />
            </div>
        </template>
        <Column header="Status" style="width: 120px">
            <template #body="slotProps">
                <Tag :value="slotProps.data.statusLabel" 
                     :severity="getStatusSeverity(slotProps.data.status)"
                     :class="{ 'dimmed-tag': missingFileIds.has(slotProps.data.id) }">
                </Tag>
            </template>
        </Column>
        <Column field="elapsed" header="Elapsed" v-if="false" style="width: 90px"></Column>
        <Column field="instant_speed" header="Speed" style="width: 110px"></Column>
        <Column field="eta" header="ETA" style="width: 80px"></Column>
        <Column field="loaded" header="Downloaded" style="width: 110px"></Column>
        <Column field="progressDisplay" header="Progress" style="width: 180px">
            <template #body="slotProps">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <div style="flex-grow: 1; height: 10px; background: #eee; border-radius: 5px; overflow: hidden;">
                         <div :style="{ width: slotProps.data.progressDisplay + '%', background: '#3b82f6', height: '100%' }"></div>
                    </div>
                    <span style="font-size: 0.8rem; min-width: 35px;">{{ slotProps.data.progressDisplay }}%</span>
                </div>
            </template>
        </Column>
        <Column style="width: 50px; text-align: center">
            <template #body="slotProps">
                <Button 
                    icon="pi pi-folder-open" 
                    text 
                    rounded 
                    severity="secondary" 
                    @click="$emit('open-folder', slotProps.data)">
                </Button>
            </template>
        </Column>
    </DataTable>
</template>

<style scoped>
.strikethrough-text {
    text-decoration: line-through;
    color: #999;
}

.dimmed-tag {
    opacity: 0.5;
    filter: grayscale(100%);
}

:deep(.file-missing-row) {
    background-color: #fcfcfc;
}

:deep(.file-missing-row) td {
    color: #9ca3af !important;
}
</style>