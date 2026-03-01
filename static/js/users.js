const UserManager = {
    allUsers: [],
    filteredUsers: [],
    currentPage: 1,
    rowsPerPage: 5,

    init() {
        this.form = document.getElementById('userForm');
        this.tableBody = document.getElementById('userTableBody');
        this.searchInput = document.getElementById('searchInput');
        this.dateStart = document.getElementById('dateStart');
        this.dateEnd = document.getElementById('dateEnd');

        this.userModal = new bootstrap.Modal(document.getElementById('userModal'));
        this.deleteModal = new bootstrap.Modal(document.getElementById('deleteModal'));

        // Botones de paginación
        this.btnFirst = document.getElementById('btnFirst');
        this.btnPrev = document.getElementById('btnPrev');
        this.btnNext = document.getElementById('btnNext');
        this.btnLast = document.getElementById('btnLast');
        this.pageIndicator = document.getElementById('pageIndicator');
        this.currentPageNum = document.getElementById('currentPageNum');

        this.bindEvents();
        this.loadUsers();
    },

    bindEvents() {
        this.form.addEventListener('submit', (e) => this.handleSubmit(e));

        // Listeners para filtros (Texto y Fechas)
        [this.searchInput, this.dateStart, this.dateEnd].forEach(el => {
            if (el) el.addEventListener('input', () => this.applyFilters());
        });

        // Eventos de paginación con saltos corregidos
        if (this.btnFirst) this.btnFirst.onclick = () => { this.currentPage = 1; this.renderTableWithPagination(); };

        if (this.btnPrev) this.btnPrev.onclick = () => {
            if (this.currentPage > 1) {
                this.currentPage--;
                this.renderTableWithPagination();
            }
        };

        if (this.btnNext) this.btnNext.onclick = () => {
            const maxPage = Math.ceil(this.filteredUsers.length / this.rowsPerPage);
            if (this.currentPage < maxPage) {
                this.currentPage++;
                this.renderTableWithPagination();
            }
        };

        if (this.btnLast) this.btnLast.onclick = () => {
            this.currentPage = Math.ceil(this.filteredUsers.length / this.rowsPerPage) || 1;
            this.renderTableWithPagination();
        };
    },

    applyFilters() {
        const term = this.searchInput.value.toLowerCase().trim();
        const start = this.dateStart.value; // YYYY-MM-DD
        const end = this.dateEnd.value;     // YYYY-MM-DD

        this.filteredUsers = this.allUsers.filter(u => {
            // 1. Filtro de Texto
            const userName = (u.usuario || "").toLowerCase();
            const userEmail = (u.email || "").toLowerCase();
            const matchesTerm = term === "" || userName.includes(term) || userEmail.includes(term);

            // 2. Filtro de Fechas (Comparamos solo YYYY-MM-DD)
            const userDateFull = u.created_at || "";
            const userDate = userDateFull.substring(0, 10);

            const matchesStart = !start || userDate >= start;
            const matchesEnd = !end || userDate <= end;

            return matchesTerm && matchesStart && matchesEnd;
        });

        this.currentPage = 1;
        this.renderTableWithPagination();
    },

    renderTableWithPagination() {
        const total = this.filteredUsers.length;
        const maxPage = Math.ceil(total / this.rowsPerPage) || 1;

        if (this.currentPage > maxPage) this.currentPage = maxPage;
        if (this.currentPage < 1) this.currentPage = 1;

        const startIdx = (this.currentPage - 1) * this.rowsPerPage;
        const pagedData = this.filteredUsers.slice(startIdx, startIdx + this.rowsPerPage);

        this.renderTable(pagedData);

        // Actualizar UI de indicadores
        if (this.pageIndicator) {
            this.pageIndicator.innerText = `Página ${this.currentPage} de ${maxPage} (${total} lectores)`;
        }

        // CORRECCIÓN: Actualiza el número dinámico 1, 2, 3... entre los botones
        if (this.currentPageNum) {
            this.currentPageNum.innerText = this.currentPage;
        }

        // Estado de botones (Habilitar/Deshabilitar)
        const isFirst = this.currentPage === 1;
        const isLast = this.currentPage === maxPage || total === 0;

        if (this.btnFirst) this.btnFirst.parentElement.classList.toggle('disabled', isFirst);
        if (this.btnPrev) this.btnPrev.parentElement.classList.toggle('disabled', isFirst);
        if (this.btnNext) this.btnNext.parentElement.classList.toggle('disabled', isLast);
        if (this.btnLast) this.btnLast.parentElement.classList.toggle('disabled', isLast);
    },

    renderTable(users) {
        if (users.length > 0) {
            this.tableBody.innerHTML = users.map(u => `
                <tr>
                    <td><strong>${u.usuario}</strong></td>
                    <td>${u.email}</td>
                    <td class="italic" style="font-size: 0.85rem;">${u.created_at || 'Reciente'}</td>
                    <td class="text-center">
                        <button class="action-btn edit-btn" onclick="UserManager.openModal(${u.id})" title="Editar Ficha">
                            <i class="fas fa-pen-fancy"></i>
                        </button>
                        <button class="action-btn delete-btn" onclick="UserManager.confirmDelete(${u.id})" title="Retirar Lector">
                            <i class="fas fa-trash"></i>
                        </button>
                    </td>
                </tr>
            `).join('');
        } else {
            this.tableBody.innerHTML = '<tr><td colspan="5" class="text-center py-4 text-muted italic">No hay registros que coincidan con la búsqueda o el rango de fechas.</td></tr>';
        }
    },

    async loadUsers() {
        this.renderSkeleton();
        try {
            const res = await fetch('/api/users');
            const data = await res.json();
            this.allUsers = Array.isArray(data) ? data : [];
            this.applyFilters();
        } catch (e) {
            this.showToast("Error al conectar con el servidor", 'error');
            this.tableBody.innerHTML = '<tr><td colspan="5" class="text-center">Error al cargar datos.</td></tr>';
        }
    },

    renderSkeleton() {
        const skeletonRow = `
            <tr>
                <td colspan="5"><div class="placeholder-glow"><span class="placeholder col-12 bg-light"></span></div></td>
            </tr>
        `;
        this.tableBody.innerHTML = skeletonRow.repeat(5);
    },

    clearFilters() {
        this.searchInput.value = "";
        this.dateStart.value = "";
        this.dateEnd.value = "";
        this.applyFilters();
    },

    async openModal(id = null) {
        this.form.reset();
        this.clearErrors();

        const modalTitle = document.getElementById('modalTitle');
        const submitBtn = this.form.querySelector('button[type="submit"]');
        const passwordLabel = document.querySelector('#passwordGroup label');

        document.getElementById('userId').value = id || "";
        modalTitle.innerText = id ? 'Editar Ficha de Lector' : 'Nueva Ficha de Lector';
        submitBtn.innerText = id ? 'Actualizar Ficha' : 'Registrar en Archivo';

        if (id) {
            try {
                const res = await fetch(`/api/users?id=${id}`);
                const u = await res.json();
                document.getElementById('userInput').value = u.usuario || "";
                document.getElementById('emailInput').value = u.email || "";
                passwordLabel.innerText = "Nueva Contraseña (opcional)";
            } catch (e) {
                this.showToast("No se pudo recuperar la ficha", 'error');
            }
        } else {
            passwordLabel.innerText = "Contraseña";
        }
        this.userModal.show();
    },

    async handleSubmit(e) {
        e.preventDefault();
        this.clearErrors();

        const submitBtn = this.form.querySelector('button[type="submit"]');
        const formData = new FormData(this.form);
        const data = Object.fromEntries(formData);

        // CORRECCIÓN: Si el id está vacío, elimínalo o ponlo como null
    // Esto evita que Rust reciba un "" cuando espera un número
    if (!data.id || data.id === "") {
        delete data.id; 
    } else {
        data.id = parseInt(data.id);
    }

        if (data.id) data.id = parseInt(data.id);
        if (data.id && !data.password) delete data.password;

        submitBtn.disabled = true;
        submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Guardando...';

        try {
            const res = await fetch('/api/users', {
                method: data.id ? 'PUT' : 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data)
            });

            const result = await res.json();

            if (result.success) {
                this.showToast(result.msg, 'success');
                this.userModal.hide();
                await this.loadUsers();
            } else {
                if (result.errors && result.errors.email) {
                    document.getElementById('error_email').innerText = result.errors.email;
                }
                this.showToast(result.msg || "Error en el formulario", 'error');
            }
        } catch (e) {
            this.showToast("Error de conexión con el servidor", 'error');
        } finally {
            submitBtn.disabled = false;
            submitBtn.innerText = data.id ? 'Actualizar Ficha' : 'Registrar en Archivo';
        }
    },

    confirmDelete(id) {
        this.userToDeleteId = id;
        this.deleteModal.show();
        document.getElementById('confirmDeleteBtn').onclick = () => this.executeDelete();
    },

    async executeDelete() {
        const id = this.userToDeleteId;
        const btn = document.getElementById('confirmDeleteBtn');
        btn.disabled = true;
        try {
            const res = await fetch('/api/users', {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ id: id })
            });
            const result = await res.json();
            if (result.success) {
                this.showToast("Lector retirado del archivo", 'warning');
                this.deleteModal.hide();
                await this.loadUsers();
            } else {
                this.showToast(result.msg, 'error');
            }
        } catch (e) {
            this.showToast("Error al eliminar", 'error');
        } finally {
            btn.disabled = false;
        }
    },

    clearErrors() {
        const errEmail = document.getElementById('error_email');
        if (errEmail) errEmail.innerText = "";
    },

    showToast(msg, type = 'success') {
        let container = document.querySelector('.toast-container');
        if (!container) {
            container = document.createElement('div');
            container.className = 'toast-container position-fixed bottom-0 end-0 p-3';
            container.style.zIndex = "2000";
            document.body.appendChild(container);
        }

        const toast = document.createElement('div');
        toast.className = `toast align-items-center text-white bg-${type === 'error' ? 'danger' : type === 'warning' ? 'warning' : 'dark'} border-0 show`;
        toast.innerHTML = `
            <div class="d-flex">
                <div class="toast-body">
                    <i class="fas fa-info-circle me-2"></i> ${msg}
                </div>
                <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast"></button>
            </div>
        `;
        container.appendChild(toast);
        setTimeout(() => toast.remove(), 4000);
    }
};

document.addEventListener('DOMContentLoaded', () => UserManager.init());