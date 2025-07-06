<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Models\Logs;
use Carbon\Carbon;
use Jenssegers\Agent\Agent;

class LogsController extends Controller
{
    public function createLog($function, $status, $message, $user, $previousValue)
    {
        $agent = new Agent();

        Logs::create([
            'user_id' => $user ? $user->id : null,
            'function' => $function,
            'url' => request()->url(),
            'previous_value' => $previousValue,
            'payload' => json_encode(request()->all()),
            'status' => $status,
            'message' => $message,
            'log_date' => Carbon::now(),
            'user_ip' => request()->ip(),
            'user_browser' => $agent->browser(),
            'user_operatingsystem' => $agent->platform(),
        ]);

    }
}
