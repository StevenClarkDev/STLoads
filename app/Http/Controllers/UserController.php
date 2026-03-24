<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Http\Controllers\Controller;
use App\Models\User;
use App\Models\UserHistory;
use Spatie\Permission\Models\Role;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Arr;
use Illuminate\Support\Facades\Hash;
use Illuminate\View\View;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Mail;


class UserController extends Controller
{
    /**
     * Display a listing of the resource.
     *
     * @return \Illuminate\Http\Response
     */
    public function index(Request $request): View
    {
        $data = User::latest()->paginate(5);

        return view('users.index', compact('data'))
            ->with('i', ($request->input('page', 1) - 1) * 5);
    }

    /**
     * Show the form for creating a new resource.
     *
     * @return \Illuminate\Http\Response
     */
    public function create(): View
    {
        $roles = Role::pluck('name', 'name')->all();

        return view('users.create', compact('roles'));
    }

    /**
     * Store a newly created resource in storage.
     *
     * @param  \Illuminate\Http\Request  $request
     * @return \Illuminate\Http\Response
     */
    public function store(Request $request): RedirectResponse
    {
        $request->validate($request, [
            'name' => 'required',
            'email' => 'required|email|unique:users,email',
            'password' => 'required|same:confirm-password',
            'roles' => 'required'
        ]);

        $input = $request->all();
        $input['password'] = Hash::make($input['password']);

        $user = User::create($input);
        $user->assignRole($request->input('roles'));

        return redirect()->route('users.index')
            ->with('success', 'User created successfully');
    }

    /**
     * Display the specified resource.
     *
     * @param  int  $id
     * @return \Illuminate\Http\Response
     */
    public function show($id): View
    {
        $user = User::find($id);

        return view('users.show', compact('user'));
    }

    /**
     * Show the form for editing the specified resource.
     *
     * @param  int  $id
     * @return \Illuminate\Http\Response
     */
    public function edit($id): View
    {
        $user = User::find($id);
        $roles = Role::pluck('name', 'name')->all();
        $userRole = $user->roles->pluck('name', 'name')->all();

        return view('users.edit', compact('user', 'roles', 'userRole'));
    }

    /**
     * Update the specified resource in storage.
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  int  $id
     * @return \Illuminate\Http\Response
     */
    public function update(Request $request, $id): RedirectResponse
    {
        $request->validate([
            'name' => 'required',
            'email' => 'required|email|unique:users,email,' . $id,
            'password' => 'same:confirm-password',
            'roles' => 'required'
        ]);

        $input = $request->all();
        if (!empty($input['password'])) {
            $input['password'] = Hash::make($input['password']);
        } else {
            $input = Arr::except($input, array('password'));
        }

        $user = User::find($id);
        $user->update($input);
        DB::table('model_has_roles')->where('model_id', $id)->delete();

        $user->assignRole($request->input('roles'));

        return redirect()->route('users.index')
            ->with('success', 'User updated successfully');
    }

    /**
     * Remove the specified resource from storage.
     *
     * @param  int  $id
     * @return \Illuminate\Http\Response
     */
    public function destroy($id): RedirectResponse
    {
        User::find($id)->delete();
        return redirect()->route('users.index')
            ->with('success', 'User deleted successfully');
    }

    public function usersByRole($id): View
    {
        $role = Role::findOrFail($id);
        $allusers = $role->users()->get();
        $users = $allusers->where('status', 1);
        // dd($users);

        return view('admin.users_by_role', compact('users', 'role'));
    }

    public function updateStatus($id, Request $request)
    {
        // Minimal validation
        $request->validate([
            'status' => 'required|in:1,2,5',          // 1=approved, 2=rejected, 5=send back
            'remarks' => 'nullable|string|max:1000',
        ]);

        // Require remarks for reject or send back
        if (in_array((int) $request->status, [2, 5]) && !$request->filled('remarks')) {
            return back()->with('error', 'Remarks are required for Reject or Send Back.');
        }

        $user = User::find($id);
        if (!$user) {
            return back()->with('error', 'User not found');
        }
        if ((int) $request->status === 2) {
            $user->rejected_at = now();
        } else if ((int) $request->status === 1) {
            $user->approved_at = now();
        }
        // Update status
        $user->status = (int) $request->status;
        $user->save();

        // Save history
        UserHistory::create([
            'user_id' => $user->id,
            'admin_id' => Auth::id(),
            'status' => (int) $request->status,
            'remarks' => $request->remarks,
        ]);

        // Send status email
        $to       = $user->email;
        $roleName = $user->roles()->first()?->name ?? 'User';

        if ((int) $request->status === 1) {
            Mail::send('emails.account_approved', [
                'name'        => $user->name,
                'role'        => $roleName,
                'approved_at' => now()->format('F j, Y'),
            ], function ($message) use ($to) {
                $message->to($to)->subject('🎉 Your STLoads Account is Approved!');
            });
        } elseif ((int) $request->status === 2) {
            Mail::send('emails.account_rejected', [
                'name'    => $user->name,
                'role'    => $roleName,
                'remarks' => $request->remarks,
            ], function ($message) use ($to) {
                $message->to($to)->subject('Your STLoads Account Application Update');
            });
        } else { // 5 = send back
            Mail::send('emails.account_revision', [
                'name'    => $user->name,
                'role'    => $roleName,
                'remarks' => $request->remarks,
            ], function ($message) use ($to) {
                $message->to($to)->subject('Action Required — Your STLoads Application');
            });
        }

        return redirect()->route('user_approval')->with('success', 'Status updated.');
    }
}
