<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Http\Controllers\Controller;
use App\Models\LoadType;
use Illuminate\View\View;
use Illuminate\Http\RedirectResponse;

class LoadTypesController extends Controller
{
    public function index(LogsController $logsController): View|RedirectResponse
    {
        try {
            $load_types = LoadType::all();
            $logsController->createLog(__METHOD__, 'success', 'User is viewing Load Types list', null, null);
            return view('load_types.index', compact('load_types'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to list Load Types: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to load data: ' . $e->getMessage());
        }
    }

    public function create(LogsController $logsController): View|RedirectResponse
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is opening Load Type create form', null, null);
            return view('load_types.create');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open Load Type create form: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open create form: ' . $e->getMessage());
        }
    }

    public function store(Request $request, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            LoadType::create([
                'name' => $request->name,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User created a new Load Type', null, null);
            return redirect()->route('load_types.index')->with('success', 'Load Type created successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create Load Type: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to create Load Type: ' . $e->getMessage());
        }
    }

    public function show($id, LogsController $logsController): View|RedirectResponse
    {
        dd('This method is not implemented yet.'); // Placeholder for future implementation
    }

    public function edit($id, LogsController $logsController): View|RedirectResponse
    {
        try {
            $load_type = LoadType::findOrFail($id);
            $logsController->createLog(__METHOD__, 'success', 'User opened edit form for Load Type (ID: ' . $id . ')', null, null);
            return view('load_types.edit', compact('load_type'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open edit form for Load Type (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open edit form: ' . $e->getMessage());
        }
    }

    public function update(Request $request, $id, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            $load_type = LoadType::findOrFail($id);
            $load_type->update([
                'name' => $request->name,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User updated Load Type (ID: ' . $id . ')', null, null);
            return redirect()->route('load_types.index')->with('success', 'Load Type updated successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to update Load Type (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to update Load Type: ' . $e->getMessage());
        }
    }

    public function destroy($id, LogsController $logsController): RedirectResponse
    {
        try {
            $load_type = LoadType::findOrFail($id);
            $load_type->delete();

            $logsController->createLog(__METHOD__, 'success', 'User deleted Load Type (ID: ' . $id . ')', null, null);
            return redirect()->route('load_types.index')->with('success', 'Load Type deleted successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to delete Load Type (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to delete Load Type: ' . $e->getMessage());
        }
    }
}
