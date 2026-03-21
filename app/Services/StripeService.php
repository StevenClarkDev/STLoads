<?php

namespace App\Services;

use Stripe\StripeClient;

class StripeService
{
    public StripeClient $client;

    public function __construct()
    {
        // Use config(), not env(), in app code
        $this->client = new StripeClient(config('services.stripe.secret'));
    }
}
