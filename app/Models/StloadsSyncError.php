<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class StloadsSyncError extends Model
{
    protected $table = 'stloads_sync_errors';

    protected $guarded = [];

    protected $casts = [
        'resolved'    => 'boolean',
        'resolved_at' => 'datetime',
    ];

    // ── Error class constants ─────────────────────────────────
    const CLASS_PUSH_FAILED           = 'push_failed';
    const CLASS_DUPLICATE_PUBLISH     = 'duplicate_publish';
    const CLASS_COMPLIANCE_BLOCKED    = 'compliance_blocked';
    const CLASS_RELEASE_GATE_FAILED   = 'release_gate_failed';
    const CLASS_DELIVERED_STILL_OPEN  = 'delivered_still_open';
    const CLASS_WITHDRAW_MISMATCH     = 'withdraw_mismatch';
    const CLASS_PAYLOAD_VALIDATION    = 'payload_validation';
    const CLASS_FINANCE_NOT_RECONCILED = 'finance_not_reconciled';

    // ── Severity constants ────────────────────────────────────
    const SEVERITY_INFO     = 'info';
    const SEVERITY_WARNING  = 'warning';
    const SEVERITY_ERROR    = 'error';
    const SEVERITY_CRITICAL = 'critical';

    // ── Relationships ─────────────────────────────────────────
    public function handoff()
    {
        return $this->belongsTo(StloadsHandoff::class, 'handoff_id');
    }

    // ── Scopes ────────────────────────────────────────────────
    public function scopeUnresolved($query)
    {
        return $query->where('resolved', false);
    }

    public function scopeBySeverity($query, string $severity)
    {
        return $query->where('severity', $severity);
    }

    // ── Helpers ───────────────────────────────────────────────
    public function resolve(string $resolvedBy, ?string $note = null): void
    {
        $this->update([
            'resolved'        => true,
            'resolved_at'     => now(),
            'resolved_by'     => $resolvedBy,
            'resolution_note' => $note,
        ]);
    }
}
