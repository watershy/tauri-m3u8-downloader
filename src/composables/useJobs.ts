import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useConfirm } from "primevue/useconfirm";
import { useToast } from "primevue/usetoast";
import { useFormatters } from './useFormatters';

export const globalTotalSpeedMb = ref(0);
export const globalActiveJobs = ref(0);

export function useJobs() {
    const confirm = useConfirm();
    const toast = useToast();
    const { formatSize, formatTime, formatEtaText, quantizeSeconds, formatStatus } = useFormatters();

    const videos = ref<any[]>([]);
    const loading = ref(false);
    const selectedJobs = ref<any[]>([]);

    const canBatchResume = computed(() => {
        return selectedJobs.value.some(j =>
            j.status !== 'Downloading' &&
            j.status !== 'Merging' &&
            j.status !== 'CompletedSuccess'
        );
    });

    const canBatchPause = computed(() => {
        return selectedJobs.value.some(j =>
            j.status === 'Merging' || j.status === 'Downloading' || j.status === 'Queued'
        );
    });

    const canBatchDelete = computed(() => selectedJobs.value.length > 0);

    const loadVideos = async () => {
        try {
            const rawVideos = await invoke<any[]>('get_jobs');
            const now = Math.floor(Date.now() / 1000);

            let currentTickSpeedBytes = 0;
            let currentTickActive = 0;

            const newVideos = rawVideos.map(v => {
                if (v.status === 'Downloading'|| v.status === 'Merging') {
                    currentTickSpeedBytes += v.instant_speed;
                    currentTickActive += 1;
                }

                const progressPercent = Math.floor(v.progress * 100);
                const endTimeOrNow = v.end_time ? v.end_time : now;
                let elapsedSeconds = endTimeOrNow - v.start_time;
                if (elapsedSeconds < 0) elapsedSeconds = 0;

                let displaySpeed = v.average_speed
                    ? `Avg ${formatSize(v.average_speed)}/s`
                    : `${formatSize(v.instant_speed)}/s`;

                return {
                    ...v,
                    size: formatSize(v.downloaded_bytes),
                    instant_speed: displaySpeed,
                    elapsed: formatTime(elapsedSeconds),
                    eta: formatEtaText(quantizeSeconds(v.eta)),
                    loaded: formatSize(v.downloaded_bytes),
                    progressDisplay: progressPercent,
                    statusLabel: formatStatus(v.status)
                };
            });

            globalTotalSpeedMb.value = currentTickSpeedBytes / (1024 * 1024);
            globalActiveJobs.value = currentTickActive;

            // Preserve selection references
            if (selectedJobs.value.length > 0) {
                const currentIds = new Set(selectedJobs.value.map(j => j.id));
                selectedJobs.value = newVideos.filter(v => currentIds.has(v.id));
            }

            videos.value = newVideos;
        } catch (error) {
            console.error("Error fetching videos:", error);
        }
    };

    const handleBatchResume = async () => {
        const targets = selectedJobs.value.filter(j =>
            j.status !== 'Downloading' &&
            j.status !== 'Merging' &&
            j.status !== 'CompletedSuccess'
        );

        if (targets.length === 0) return;

        for (const job of targets) {
            try {
                await invoke('resume_job', { jobId: job.id });
            } catch (e) {
                console.error(`Failed to resume ${job.file_name}:`, e);
            }
        }
        await loadVideos();
        toast.add({ severity: 'success', summary: 'Batch Resume', detail: `Resumed ${targets.length} downloads`, life: 3000 });
    };

    const handleBatchPause = async () => {
        const targets = selectedJobs.value.filter(j =>
            j.status === 'Downloading' || j.status === 'Merging' || j.status === 'Queued'
        );

        for (const job of targets) {
            try {
                await invoke('pause_job', { jobId: job.id });
            } catch (e) {
                console.error(`Failed to pause ${job.file_name}:`, e);
            }
        }
        await loadVideos();
    };

    const handleBatchDelete = () => {
        const count = selectedJobs.value.length;
        if (count === 0) return;

        const runningCount = selectedJobs.value.filter(j =>
            j.status === 'Downloading' || j.status === 'Merging'
        ).length;

        let header = 'Batch Delete';
        let message = `Are you sure you want to delete ${count} selected items?`;
        let icon = 'pi pi-trash';
        let acceptLabel = 'Delete';

        if (runningCount > 0) {
            header = 'Stop & Delete Active Jobs?';
            message = `You have selected ${runningCount} ACTIVE downloads.\n\nProceeding will STOP them immediately and PERMANENTLY DELETE their files.\n\nAre you sure you want to delete all ${count} items?`;
            icon = 'pi pi-exclamation-triangle';
            acceptLabel = 'Stop All & Delete';
        }

        confirm.require({
            message, header, icon, acceptLabel,
            acceptClass: 'p-button-danger',
            rejectClass: 'p-button-secondary',
            rejectLabel: 'Cancel',
            accept: async () => {
                for (const job of selectedJobs.value) {
                    try {
                        await invoke('delete_job', { jobId: job.id });
                    } catch (e) {
                        console.error(`Failed to delete ${job.id}:`, e);
                    }
                }
                selectedJobs.value = [];
                await loadVideos();
                toast.add({ severity: 'success', summary: 'Deleted', detail: `${count} items removed`, life: 3000 });
            }
        });
    };

    const handleDeleteCompleted = async () => {
        // confirm.require({
        //     message: 'Are you sure you want to remove all completed and failed jobs from the list?',
        //     header: 'Clear History',
        //     icon: 'pi pi-trash',
        //     acceptClass: 'p-button-danger',
        //     rejectClass: 'p-button-secondary',
        //     acceptLabel: 'Clear All',
        //     rejectLabel: 'Cancel',
        //     accept: async () => {
        //         try {
        //             await invoke('delete_completed_jobs');
        //             loadVideos();
        //         } catch (e) {
        //             console.error("Failed to clear jobs:", e);
        //         }
        //     }
        // });
        try {
            await invoke('delete_completed_jobs');
            loadVideos();
        } catch (e) {
            console.error("Failed to clear jobs:", e);
        }
    };

    return {
        videos,
        loading,
        selectedJobs,
        // Computed
        canBatchResume,
        canBatchPause,
        canBatchDelete,
        // Actions
        loadVideos,
        handleBatchResume,
        handleBatchPause,
        handleBatchDelete,
        handleDeleteCompleted
    };
}