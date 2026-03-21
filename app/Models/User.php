<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Spatie\Permission\Traits\HasRoles;
use Laravel\Sanctum\HasApiTokens;

class User extends Authenticatable
{
    /** @use HasFactory<\Database\Factories\UserFactory> */
    use HasFactory, Notifiable, HasRoles, HasApiTokens;

    /**
     * The attributes that are mass assignable.
     *
     * @var list<string>
     */
    protected $fillable = [
        'name',
        'email',
        'password',
        'role_id',
        'dob',
        'gender',
        'phone_no',
        'ucr_hcc_no',
        'mc_cbsa_usdot_no',
        'cnic_no',
        'address',
        'otp',
        'otp_expires_at',
        'otp_resend_count',
        'last_otp_resend_at',
        'image',
        'email_verified_at',
        'status',
        'approved_at',
        'rejected_at',
        'kyc_pending_at',
        'stripe_connect_account_id',
        'payouts_enabled',
        'kyc_status',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var list<string>
     */
    protected $hidden = [
        'password',
        'remember_token',
    ];

    /**
     * Get the attributes that should be cast.
     *
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'email_verified_at' => 'datetime',
            'password' => 'hashed',
        ];
    }

    public function carrierPreference()
    {
        return $this->hasOne(CarrierPreference::class, 'user_id');
    }
    public function kycDocuments()
    {
        return $this->hasMany(KycDocuments::class, 'user_id');
    }
    public function history()
    {
        return $this->hasMany(UserHistory::class, 'user_id');
    }
    public function latestHistory()
    {
        return $this->hasOne(UserHistory::class, 'user_id')->latestOfMany(); // uses created_at/id
    }
}
