@extends('admin-layout.app')

@section('content')
    <div class="row">
        <div class="col-xl-12 box-col-6 px-3 py-2">
            <div class="card">
                <div class="card-body p-0">
                    <div class="card mx-3">
                        <div class="card-header pb-0 card-no-border">
                            <h4>Roles List</h4>
                            <span>See roles below.</span>
                        </div>
                        <div class="card-body">
                            <div class="table-responsive">
                                <div style="max-height:500px; overflow-y:auto;">
                                    <table class="table table-striped w-100" id="user-approval-table">
                                        <thead style="position: sticky; top: 0; background: #fff; z-index: 2;">
                                            <tr>
                                                <th>S No.</th>
                                                <th>Name</th>
                                                <th>Count</th>
                                                <th>Action</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            @foreach ($roles as $key => $role)
                                                <tr>
                                                    <td>{{ ++$i }}</td>
                                                    <td>{{ $role->name }}</td>
                                                    <td>
                                                        <span class="badge rounded-pill bg-primary py-2 px-3">
                                                            {{ $role->users->count() }}
                                                        </span>
                                                    </td>
                                                    <td>
                                                        <a class="btn btn-info btn-sm"
                                                            href="{{ route('roles.show', $role->id) }}">
                                                            <i class="fa fa-list"></i> Show
                                                        </a>
                                                        @can('role-edit')
                                                            <a class="btn btn-primary btn-sm"
                                                                href="{{ route('roles.edit', $role->id) }}">
                                                                <i class="fa fa-pencil"></i> Edit
                                                            </a>
                                                        @endcan
                                                        {{-- @can('role-delete')
                                                        <form method="POST" action="{{ route('roles.destroy', $role->id) }}"
                                                            style="display:inline">
                                                            @csrf
                                                            @method('DELETE')
                                                            <button type="submit" class="btn btn-danger btn-sm">
                                                                <i class="fa-solid fa-trash"></i> Delete
                                                            </button>
                                                        </form>
                                                        @endcan --}}
                                                    </td>
                                                </tr>
                                            @endforeach
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            <div class="d-flex justify-content-between align-items-center mt-3">
                                <div class="d-flex align-items-center gap-2">
                                    <label class="mb-0 text-muted small">Show:</label>
                                    <select id="perPageSelect" class="form-select form-select-sm" style="width: auto;" onchange="changePerPage(this.value)">
                                        <option value="10" {{ request('per_page', 10) == 10 ? 'selected' : '' }}>10</option>
                                        <option value="20" {{ request('per_page', 10) == 20 ? 'selected' : '' }}>20</option>
                                        <option value="50" {{ request('per_page', 10) == 50 ? 'selected' : '' }}>50</option>
                                        <option value="100" {{ request('per_page', 10) == 100 ? 'selected' : '' }}>100</option>
                                    </select>
                                    <span class="text-muted small">entries per page</span>
                                </div>
                                <div>
                                    {!! $roles->appends(['per_page' => request('per_page', 10)])->links('pagination::bootstrap-5') !!}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

<script>
    function changePerPage(value) {
        const url = new URL(window.location.href);
        url.searchParams.set('per_page', value);
        url.searchParams.delete('page'); // Reset to page 1 when changing per_page
        window.location.href = url.toString();
    }
</script>
@endsection
