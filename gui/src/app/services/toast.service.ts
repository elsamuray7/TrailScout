import { Injectable, TemplateRef } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class ToastService {
  toasts: any[] = [];

  show(textOrTpl: string | TemplateRef<any>, options: any = {}) {
    this.toasts.push({ textOrTpl, ...options });
  }

  remove(toast: any) {
    this.toasts = this.toasts.filter(t => t !== toast);
  }

  clear() {
    this.toasts.splice(0, this.toasts.length);
  }

  showStandard(message: string) {
    this.show(message);
  }

  showSuccess(message: string) {
    this.show(message, { classname: 'bg-success'});
  }

  showDanger(message: string) {
    this.show(message, { classname: 'bg-danger'});
  }
}
