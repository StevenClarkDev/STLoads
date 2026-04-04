<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class StloadsHandoff extends Model
{
    protected $table = 'stloads_handoffs';

    protected $guarded = [];

    protected $casts = [
        'temperature_data'            => 'array',
        'container_data'              => 'array',
        'securement_data'             => 'array',
        'accessorial_flags'           => 'array',
        'compliance_summary'          => 'array',
        'required_documents_status'   => 'array',
        'raw_payload'                 => 'array',
        'is_hazardous'                => 'boolean',
        'compliance_passed'           => 'boolean',
        'pickup_window_start'         => 'datetime',
        'pickup_window_end'           => 'datetime',
        'dropoff_window_start'        => 'datetime',
        'dropoff_window_end'          => 'datetime',
        'queued_at'                   => 'datetime',
        'published_at'                => 'datetime',
        'withdrawn_at'                => 'datetime',
        'closed_at'                   => 'datetime',
        'tms_status_at'               => 'datetime',
        'last_webhook_at'             => 'datetime',
    ];

    // ── Status constants ──────────────────────────────────────
    const STATUS_QUEUED             = 'queued';
    const STATUS_PUSH_IN_PROGRESS   = 'push_in_progress';
    const STATUS_PUBLISHED          = 'published';
    const STATUS_PUSH_FAILED        = 'push_failed';
    const STATUS_REQUEUE_REQUIRED   = 'requeue_required';
    const STATUS_WITHDRAWN          = 'withdrawn';
    const STATUS_CLOSED             = 'closed';

    // ── TMS dispatch status constants ─────────────────────────
    const TMS_DISPATCHED    = 'dispatched';
    const TMS_IN_TRANSIT    = 'in_transit';
    const TMS_AT_PICKUP     = 'at_pickup';
    const TMS_AT_DELIVERY   = 'at_delivery';
    const TMS_DELIVERED     = 'delivered';
    const TMS_CANCELLED     = 'cancelled';
    const TMS_INVOICED      = 'invoiced';
    const TMS_SETTLED       = 'settled';

    // ── Relationships ─────────────────────────────────────────
    public function load()
    {
        return $this->belongsTo(Load::class, 'load_id');
    }

    public function events()
    {
        return $this->hasMany(StloadsHandoffEvent::class, 'handoff_id');
    }

    public function externalRefs()
    {
        return $this->hasMany(StloadsExternalRef::class, 'handoff_id');
    }

    public function syncErrors()
    {
        return $this->hasMany(StloadsSyncError::class, 'handoff_id');
    }

    public function reconciliationLogs()
    {
        return $this->hasMany(StloadsReconciliationLog::class, 'handoff_id');
    }

    // ── Helpers ───────────────────────────────────────────────
    public function recordEvent(string $type, ?string $performedBy = null, ?string $module = null, $result = null): StloadsHandoffEvent
    {
        return $this->events()->create([
            'event_type'       => $type,
            'performed_by'     => $performedBy,
            'source_module'    => $module,
            'payload_snapshot' => json_encode($this->raw_payload),
            'result'           => $result,
        ]);
    }

    public function markPublished(int $loadId): void
    {
        $this->update([
            'status'       => self::STATUS_PUBLISHED,
            'load_id'      => $loadId,
            'published_at' => now(),
            'last_push_result' => 'success',
        ]);
    }

    public function markFailed(string $reason): void
    {
        $this->update([
            'status'           => self::STATUS_PUSH_FAILED,
            'retry_count'      => $this->retry_count + 1,
            'last_push_result' => $reason,
        ]);
    }
}
