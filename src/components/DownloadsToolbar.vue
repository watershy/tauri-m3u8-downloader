<script setup lang="ts">
import Button from 'primevue/button';

defineProps<{
    canResume: boolean;
    canPause: boolean;
    canDelete: boolean;
}>();

const emit = defineEmits<{
    (e: 'new-job'): void;
    (e: 'resume'): void;
    (e: 'pause'): void;
    (e: 'delete'): void;
    (e: 'clear-completed'): void;
}>();
</script>

<template>
    <div class="toolbar mb-3">
        <div class="group-left">
            <Button label="New Job" icon="pi pi-plus" severity="success" @click="$emit('new-job')" />
            <span class="divider"></span>
            <Button label="Start" icon="pi pi-play" severity="info" :disabled="!canResume" @click="$emit('resume')" />
            <Button label="Pause" icon="pi pi-pause" severity="warning" :disabled="!canPause" @click="$emit('pause')" />
        </div>
        <div class="group-right">
            <Button label="Delete" icon="pi pi-trash" severity="danger" outlined :disabled="!canDelete" @click="$emit('delete')" />
            <Button label="Clear Completed" icon="pi pi-times" severity="secondary" text @click="$emit('clear-completed')" />
        </div>
    </div>
</template>

<style scoped>
/* Paste toolbar CSS here */
.toolbar { display: flex; justify-content: space-between; align-items: center; background: #f8f9fa; padding: 0.5rem; border-radius: 6px; border: 1px solid #e5e7eb; }
.group-left, .group-right { display: flex; gap: 0.5rem; align-items: center; }
.divider { width: 1px; height: 24px; background-color: #d1d5db; margin: 0 0.5rem; }
</style>