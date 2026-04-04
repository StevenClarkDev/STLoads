<?php

use App\Services\StloadsReconciler;
use Illuminate\Foundation\Inspiring;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\Schedule;

Artisan::command('inspire', function () {
    $this->comment(Inspiring::quote());
})->purpose('Display an inspiring quote');

Artisan::command('stloads:reconcile', function () {
    $this->info('Running STLOADS reconciliation scan...');
    $results = StloadsReconciler::runReconciliationScan();
    $total = array_sum($results);
    if ($total > 0) {
        $this->warn("Found {$total} items:");
        foreach ($results as $key => $count) {
            if ($count > 0) {
                $this->line("  {$key}: {$count}");
            }
        }
    } else {
        $this->info('No issues found.');
    }
})->purpose('Run STLOADS reconciliation scan to detect and fix TMS/STLOADS mismatches');

// Schedule the reconciliation scan to run every 6 hours
Schedule::command('stloads:reconcile')->everySixHours();
