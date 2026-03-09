<script setup lang="ts">
import { globalActiveJobs } from '../composables/useJobs';
import { useThroughputGraph } from '../composables/useThroughputGraph';

// Initialize our graph engine and extract the calculated data
const { smartMetrics, graphPath } = useThroughputGraph(60);
</script>

<template>
    <footer class="status-bar">
        <div class="metrics">
            <div class="metric-item">
                <i class="pi pi-cloud-download"></i>
                <span class="label">Total Speed:</span>
                <span class="value">{{ smartMetrics.totalSpeedLabel }}</span>
            </div>
            <div class="divider"></div>
            <div class="metric-item">
                <i class="pi pi-list"></i>
                <span class="label">Active Jobs:</span>
                <span class="value">{{ globalActiveJobs }}</span>
            </div>
        </div>

        <div class="graph-wrapper" title="Network Throughput">
            <div class="graph-y-axis">{{ smartMetrics.maxLabel }}</div>
            <div class="graph-container">
                <div class="graph-instant-speed">{{ smartMetrics.instantLabel }}</div>
                <svg class="sparkline" viewBox="0 0 100 40" preserveAspectRatio="none">
                    <path :d="graphPath" class="sparkline-area" />
                </svg>
            </div>
        </div>
    </footer>
</template>

<style scoped>
.status-bar { height: 40px; background-color: #f8f9fa; border-top: 1px solid #e5e7eb; display: flex; justify-content: space-between; align-items: center; padding: 0 15px; color: #6b7280; font-size: 0.8rem; flex-shrink: 0; }
.metrics { display: flex; align-items: center; gap: 15px; }
.metric-item { display: flex; align-items: center; gap: 6px; }
.metric-item i { color: #9ca3af; font-size: 0.8rem; }
.label { color: #6b7280; }
.value { color: #111827; font-weight: 600; font-family: monospace; font-size: 0.85rem; }
.divider { width: 1px; height: 16px; background-color: #d1d5db; }
.graph-wrapper { display: flex; align-items: flex-start; gap: 6px; }
.graph-y-axis { font-size: 0.6rem; color: #9ca3af; font-family: monospace; line-height: 1; margin-top: -1px; white-space: nowrap; }
.graph-container { position: relative; height: 24px; width: 150px; background-color: #ffffff; border: 1px solid #e5e7eb; border-radius: 3px; overflow: hidden; }
.graph-instant-speed { position: absolute; top: 1px; left: 4px; font-size: 0.6rem; color: #10b981; font-family: monospace; z-index: 10; pointer-events: none; }
.sparkline { width: 100%; height: 100%; display: block; }
.sparkline-area { fill: rgba(16, 185, 129, 0.15); stroke: rgb(16, 185, 129); stroke-width: 1.5; stroke-linejoin: round; }
</style>