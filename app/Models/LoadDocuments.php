<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model as Models;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Http\UploadedFile;
use Illuminate\Support\Facades\Storage;

class LoadDocuments extends Models
{
    use HasFactory;
    protected $table = 'load_documents';
    protected $guarded = [];
    public $timestamps = true;

    public const TYPE_STANDARD   = 'standard';
    public const TYPE_BLOCKCHAIN = 'blockchain';

    /** @var array<string> */
    public const ALLOWED_TYPES = [
        self::TYPE_STANDARD,
        self::TYPE_BLOCKCHAIN,
    ];

    /** Mass-assignable fields */
    protected $fillable = [
        'load_id',
        'document_name',
        'document_type',
        'file_path',
        'original_name',
        'mime_type',
        'file_size',
        // blockchain (mock) fields
        'hash',
        'hash_algorithm',
        'mock_blockchain_tx',
        'mock_blockchain_timestamp',
    ];

    /** Attribute casting */
    protected $casts = [
        'file_size'                => 'integer',
        'mock_blockchain_timestamp' => 'datetime',
    ];

    /** Appended accessors (e.g., $doc->file_url) */
    protected $appends = [
        'file_url',
    ];


    public function load_master()
    {
        return $this->belongsTo(Load::class);
    }

    /* ----------------------------- Scopes ----------------------------- */

    public function scopeBlockchain($query)
    {
        return $query->where('document_type', self::TYPE_BLOCKCHAIN);
    }

    public function scopeForLoad($query, int $loadId)
    {
        return $query->where('load_id', $loadId);
    }

    /* ---------------------------- Accessors --------------------------- */

    public function getFileUrlAttribute(): ?string
    {
        if (!$this->file_path) {
            return null;
        }
        // Assumes you stored on 'public' disk and ran `php artisan storage:link`
        return $this->file_path ? Storage::url($this->file_path) : null;
    }

    public function getIsBlockchainAttribute(): bool
    {
        return $this->document_type === self::TYPE_BLOCKCHAIN;
    }

    /* ---------------------------- Mutators ---------------------------- */

    public function setDocumentTypeAttribute($value): void
    {
        $this->attributes['document_type'] = strtolower(trim($value));
    }

    public function setDocumentNameAttribute($value): void
    {
        $this->attributes['document_name'] = trim($value);
    }

    /* ------------------------ Helper / Utilities ---------------------- */

    /**
     * Compute a hash over a local file path using the provided algorithm.
     * Default is sha256 to match your controller.
     */
    public static function hashFilePath(string $absolutePath, string $algorithm = 'sha256'): string
    {
        return hash_file($algorithm, $absolutePath);
    }

    /**
     * Compute a hash over an uploaded file (tmp path) using the provided algorithm.
     */
    public static function hashUploadedFile(UploadedFile $file, string $algorithm = 'sha256'): string
    {
        return hash_file($algorithm, $file->getRealPath());
    }

    /**
     * Find a blockchain-anchored document by its content hash (exact match).
     * Returns the matching record or null.
     */
    public static function findByHash(string $hash)
    {
        return static::query()
            ->blockchain()
            ->where('hash', $hash)
            ->first();
    }

    /**
     * Convenience: verify if this record’s stored hash matches the given hash.
     */
    public function matchesHash(string $hash): bool
    {
        return $this->hash && hash_equals($this->hash, $hash);
    }
}
