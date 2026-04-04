<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class StloadsReconciliationLog extends Model
{
    protected $table = 'stloads_reconciliation_log';

    protected $guarded = [];

    protected $casts = [
        'webhook_payload' => 'array',
    ];

    // ── Action constants ──────────────────────────────────────
    const ACTION_STATUS_UPDATE    = 'status_update';
    const ACTION_AUTO_WITHDRAW    = 'auto_withdraw';
    const ACTION_AUTO_CLOSE       = 'auto_close';
    const ACTION_AUTO_ARCHIVE     = 'auto_archive';
    const ACTION_RATE_UPDATE      = 'rate_update';
    const ACTION_MISMATCH_DETECTED = 'mismatch_detected';
    const ACTION_FORCE_SYNC       = 'force_sync';

    // ── Relationships ─────────────────────────────────────────
    public function handoff()
    {
        return $this->belongsTo(StloadsHandoff::class, 'handoff_id');
    }
}
