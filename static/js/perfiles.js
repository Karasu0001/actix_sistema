const PerfilManager = {
    allPerfiles: [],
    filteredPerfiles: [],
    currentPage: 1,
    rowsPerPage: 5,

    init() {
        this.tableBody = document.getElementById('perfilTableBody');
        this.form = document.getElementById('perfilForm');
        this.modal = document.getElementById('perfilModal');
        this.deleteModal = document.getElementById('deleteModal');

        this.searchInput = document.getElementById('searchInput');
        this.btnClear = document.getElementById('btnClearFilters');
        this.btnNew = document.getElementById('btnNewPerfil');

        this.btnFirst = document.getElementById('btnFirst');
        this.btnPrev = document.getElementById('btnPrev');
        this.btnNext = document.getElementById('btnNext');
        this.btnLast = document.getElementById('btnLast');
        this.pageIndicator = document.getElementById('currentPageIndicator');

        // Lógica de permisos
        if (this.btnNew && typeof PERMISOS_MODULO !== 'undefined') {
            if (!PERMISOS_MODULO.canAdd) this.btnNew.style.display = 'none';
        }

        this.bindEvents();
        this.loadPerfiles();
    },

    bindEvents() {
        this.form.onsubmit = (e) => this.handleSubmit(e);

        if (this.searchInput) {
            this.searchInput.addEventListener('input', () => this.applyFilters());
        }

        if (this.btnClear) {
            this.btnClear.onclick = () => {
                this.searchInput.value = "";
                this.applyFilters();
            };
        }

        if (this.btnFirst) this.btnFirst.onclick = () => { this.currentPage = 1; this.renderTableWithPagination(); };
        if (this.btnPrev) this.btnPrev.onclick = () => { if (this.currentPage > 1) { this.currentPage--; this.renderTableWithPagination(); } };
        if (this.btnNext) this.btnNext.onclick = () => {
            const maxPage = Math.ceil(this.filteredPerfiles.length / this.rowsPerPage);
            if (this.currentPage < maxPage) { this.currentPage++; this.renderTableWithPagination(); }
        };
        if (this.btnLast) this.btnLast.onclick = () => {
            this.currentPage = Math.ceil(this.filteredPerfiles.length / this.rowsPerPage) || 1;
            this.renderTableWithPagination();
        };
    },

    async loadPerfiles() {
        try {
            // URL ajustada a tu router: /api/perfiles
            const res = await fetch('/api/perfiles');
            const data = await res.json();
            this.allPerfiles = Array.isArray(data) ? data : [];
            this.applyFilters();
        } catch (e) {
            this.showToast("Error al cargar perfiles", 'error');
        }
    },

    applyFilters() {
        const term = this.searchInput.value.toLowerCase().trim();
        this.filteredPerfiles = this.allPerfiles.filter(p => {
            const name = (p.strNombrePerfil || "").toLowerCase();
            return term === "" || name.includes(term);
        });
        this.currentPage = 1;
        this.renderTableWithPagination();
    },

    renderTableWithPagination() {
        const total = this.filteredPerfiles.length;
        const maxPage = Math.ceil(total / this.rowsPerPage) || 1;
        if (this.currentPage > maxPage) this.currentPage = maxPage;

        const start = (this.currentPage - 1) * this.rowsPerPage;
        const end = start + this.rowsPerPage;
        const pagedData = this.filteredPerfiles.slice(start, end);

        this.renderTable(pagedData);

        if (this.pageIndicator) {
            this.pageIndicator.innerText = `Página ${this.currentPage} de ${maxPage}`;
        }

        if (this.btnFirst) {
            this.btnFirst.disabled = this.currentPage === 1;
            this.btnPrev.disabled = this.currentPage === 1;
            this.btnNext.disabled = this.currentPage === maxPage || total === 0;
            this.btnLast.disabled = this.currentPage === maxPage || total === 0;
        }
    },

    renderTable(data) {
        if (data.length > 0) {
            this.tableBody.innerHTML = data.map(p => {
                let botonesAccion = '';
                if (typeof PERMISOS_MODULO !== 'undefined') {
                    if (PERMISOS_MODULO.canEdit) {
                        botonesAccion += `<button class="btn-edit" title="Editar" onclick="PerfilManager.openModal(${p.id})"><i class="fas fa-edit"></i></button>`;
                    }
                    if (PERMISOS_MODULO.canDelete) {
                        botonesAccion += `<button class="btn-delete" title="Eliminar" onclick="PerfilManager.confirmDelete(${p.id})"><i class="fas fa-trash"></i></button>`;
                    }
                }

                if (!botonesAccion) botonesAccion = '<span class="text-muted small">Sin permisos</span>';

                return `
                <tr>
                    <td style="font-weight: 600; color: #1e293b;">${p.strNombrePerfil}</td> 
    <td>
                        <span class="badge" 
                              style="padding: 4px 12px; border-radius: 20px; font-weight: 700; font-size: 11px; 
                              background: ${p.bitAdministrador ? '#10b981' : '#cbd5e0'}; color: white;">
                            ${p.bitAdministrador ? 'ADMIN' : 'USUARIO'}
                        </span>
                    </td>
                    <td style="text-align: center;">
                        <div class="table-actions">${botonesAccion}</div>
                    </td>
                </tr>
            `;
            }).join('');
        } else {
            this.tableBody.innerHTML = '<tr><td colspan="3" style="text-align:center; padding: 20px;">No se encontraron perfiles.</td></tr>';
        }
    },

    async openModal(id = null) {
        this.form.reset();
        document.getElementById('perfilId').value = id || ""; // ID oculto

        const modalTitle = document.getElementById('modalTitle');
        const submitBtn = this.form.querySelector('button[type="submit"]');

        submitBtn.disabled = false;
        submitBtn.style.opacity = "1";
        submitBtn.innerHTML = id ? '<span>Guardar Cambios</span> <i class="fas fa-check"></i>' : '<span>Crear Perfil</span> <i class="fas fa-plus"></i>';
        modalTitle.innerText = id ? 'Editar Perfil' : 'Nuevo Perfil';

        if (id) {
            try {
                const res = await fetch(`/api/perfiles/${id}`);
                const p = await res.json();

                console.log("🔍 DATA:", p);

                // ✅ CORRECTO
                this.form.querySelector("#strNombrePerfil").value = p.strNombrePerfil;
                this.form.querySelector("#bitAdministrador").checked = p.bitAdministrador;

            } catch (e) {
                this.showToast("Error al obtener datos", 'error');
            }
        }
        this.modal.style.display = 'flex';
    },

    closeModal() { this.modal.style.display = 'none'; },

    confirmDelete(id) {
        this.perfilToDeleteId = id;
        this.deleteModal.style.display = 'flex';
        const confirmBtn = document.getElementById('confirmDeleteBtn');
        confirmBtn.disabled = false;
        confirmBtn.onclick = () => this.executeDelete();
    },

    closeDeleteModal() { this.deleteModal.style.display = 'none'; },

    async executeDelete() {
        const confirmBtn = document.getElementById('confirmDeleteBtn');
        confirmBtn.disabled = true;
        confirmBtn.innerText = "Eliminando...";

        try {
            // URL ajustada a: /api/perfiles/{id}
            const res = await fetch(`/api/perfiles/${this.perfilToDeleteId}`, {
                method: 'DELETE'
            });
            const result = await res.json();
            if (result.success) {
                this.showToast("Perfil eliminado correctamente", 'warning');
                this.loadPerfiles();
            } else {
                this.showToast(result.msg, 'error');
            }
        } catch (e) {
            this.showToast("Error al eliminar", 'error');
        } finally {
            this.closeDeleteModal();
        }
    },

    async handleSubmit(e) {
        e.preventDefault();
        const submitBtn = this.form.querySelector('button[type="submit"]');
        const originalContent = submitBtn.innerHTML;

        submitBtn.disabled = true;
        submitBtn.style.opacity = "0.7";
        submitBtn.innerHTML = '<span>Guardando...</span> <i class="fas fa-spinner fa-spin"></i>';

        // Recopilamos datos para enviar como JSON (Recomendado para Rust)
        const idValue = document.getElementById('perfilId').value;
        const payload = {
            id: idValue ? parseInt(idValue) : null,
            strNombrePerfil: this.form.strNombrePerfil.value, // 🔥 corregido
            bitAdministrador: this.form.bitAdministrador.checked
        };

        try {
            // Siempre usamos POST /api/perfiles, el Service de Rust decide si hace INSERT o UPDATE basado en el ID
            const res = await fetch('/api/perfiles', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            const result = await res.json();

            if (result.success) {
                this.showToast(payload.id ? "Perfil actualizado" : "Perfil creado", 'success');
                this.closeModal();
                this.loadPerfiles();
            } else {
                this.showToast(result.msg || "Error al procesar", 'error');
            }
        } catch (e) {
            this.showToast("Error de conexión al guardar", 'error');
        } finally {
            submitBtn.disabled = false;
            submitBtn.style.opacity = "1";
            submitBtn.innerHTML = originalContent;
        }
    },

    showToast(msg, type = 'success') {
        let container = document.querySelector('.toast-container');
        if (!container) {
            container = document.createElement('div');
            container.className = 'toast-container';
            document.body.appendChild(container);
        }

        const config = {
            success: { icon: 'fa-check-circle', title: 'Éxito' },
            error: { icon: 'fa-times-circle', title: 'Error' },
            warning: { icon: 'fa-exclamation-triangle', title: 'Atención' }
        };

        const { icon, title } = config[type] || config.success;
        const toast = document.createElement('div');
        toast.className = `toast ${type}`;
        toast.innerHTML = `
            <i class="fas ${icon}"></i>
            <div class="toast-content">
                <span class="toast-title">${title}</span>
                <span class="toast-message">${msg}</span>
            </div>
        `;

        container.appendChild(toast);
        setTimeout(() => {
            toast.style.opacity = '0';
            setTimeout(() => toast.remove(), 400);
        }, 3500);
    }
};

document.addEventListener('DOMContentLoaded', () => PerfilManager.init());